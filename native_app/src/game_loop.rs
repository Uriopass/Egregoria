use std::ptr::addr_of;
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use common::history::History;
use engine::{Context, FrameContext, MeshBuilder};
use geom::{vec2, vec3, Camera, LinearColor};
use simulation::Simulation;

use crate::audio::GameAudio;
use crate::debug_gui::debug_window::DebugObjs;
use crate::debug_gui::render_oldgui;
use crate::gui;
use crate::gui::follow::FollowEntity;
use crate::gui::keybinds::KeybindState;
use crate::gui::terraforming::TerraformingResource;
use crate::gui::toolbox::building;
use crate::gui::windows::settings::{manage_settings, Settings};
use crate::gui::UiTextures;
use crate::gui::{render_newgui, ExitState, GuiState, TimeAlways, Tool};
use crate::inputmap::{Bindings, InputAction, InputMap};
use crate::rendering::{InstancedRender, MapRenderOptions, MapRenderer, OrbitCamera};
use crate::uiworld::{SaveLoadState, UiWorld};
use prototypes::GameTime;
use simulation::utils::scheduler::SeqSchedule;

pub const VERSION: &str = include_str!("../../VERSION");

/// State is the main struct that contains all the state of the game and game UI.
pub struct State {
    pub sim: Arc<RwLock<Simulation>>,
    pub uiw: UiWorld,
    pub game_schedule: SeqSchedule,

    instanced_renderer: InstancedRender,
    map_renderer: MapRenderer,
    immediate_renderer: MeshBuilder<true>,

    all_audio: GameAudio,
}

impl engine::framework::State for State {
    fn new(ctx: &mut Context) -> Self {
        profiling::scope!("game_loop::init");
        goryak::set_blur_texture(ctx.yakui.blur_bg_texture);

        let camera = OrbitCamera::load((ctx.gfx.size.0, ctx.gfx.size.1));

        log::info!("loaded egui_render");

        let sim: Simulation =
            Simulation::load_from_disk("world").unwrap_or_else(|| Simulation::new(true));
        let game_schedule = Simulation::schedule();
        let mut uiworld = UiWorld::init();

        let mut bindings = uiworld.write::<Bindings>();
        let default_bindings = Bindings::default();
        bindings
            .0
            .retain(|act, _| default_bindings.0.contains_key(act));
        for (act, comb) in default_bindings.0 {
            bindings.0.entry(act).or_insert(comb);
        }
        uiworld.write::<InputMap>().build_input_tree(&mut bindings);
        drop(bindings);

        uiworld.insert(UiTextures::new(&mut ctx.gfx, &mut ctx.yakui));

        uiworld.insert(camera.camera);
        uiworld.insert(camera);

        log::info!("version is {}", VERSION);

        {
            let s = uiworld.read::<Settings>();
            manage_settings(ctx, &s);
        }

        defer!(log::info!("finished init of game loop"));
        building::do_icons(ctx, &uiworld);

        let me = Self {
            uiw: uiworld,
            game_schedule,
            instanced_renderer: InstancedRender::new(&mut ctx.gfx),
            map_renderer: MapRenderer::new(&mut ctx.gfx, &sim),
            all_audio: GameAudio::new(&mut ctx.audio),
            sim: Arc::new(RwLock::new(sim)),
            immediate_renderer: MeshBuilder::new(ctx.gfx.tess_material),
        };
        me.sim.write().unwrap().map().dispatch_all();
        me
    }

    fn update(&mut self, ctx: &mut Context) {
        profiling::scope!("game_loop::update");
        self.uiw.write::<TimeAlways>().0 += ctx.delta;

        {
            let mut timings = self.uiw.write::<Timings>();
            timings.engine_render_time.add_value(ctx.times.render_time);
            timings.gui_time.add_value(ctx.times.gui_time);
            timings.total_cpu_time.add_value(ctx.times.total_cpu_time);
        }

        let mut slstate = self.uiw.write::<SaveLoadState>();
        if slstate.please_save && !slstate.saving_status.load(Ordering::SeqCst) {
            slstate.please_save = false;
            let cpy = self.sim.clone();
            slstate.saving_status.store(true, Ordering::SeqCst);
            let status = slstate.saving_status.clone();
            std::thread::spawn(move || {
                profiling::scope!("game_loop::update::save");
                cpy.read().unwrap().save_to_disk("world");
                status.store(false, Ordering::SeqCst);
            });
        }
        drop(slstate);

        crate::network::sim_update(self);

        if std::mem::take(&mut self.uiw.write::<SaveLoadState>().render_reset) {
            self.reset(ctx);
        }

        if !ctx.egui.last_mouse_captured {
            let sim = self.sim.read().unwrap();
            let map = sim.map();
            let ray = self
                .uiw
                .read::<OrbitCamera>()
                .camera
                .unproj_ray(ctx.input.mouse.screen);
            self.uiw.write::<InputMap>().ray = ray;

            if let Some(ray) = ray {
                let cast = map.environment.raycast(ray);

                let inp = &mut self.uiw.write::<InputMap>();

                inp.unprojected = cast.map(|(mut proj, _)| {
                    proj.z = proj.z.max(0.0);
                    proj
                });
                inp.unprojected_normal = cast.map(|x| x.1);
            }
        }

        ctx.keybind_mode = self.uiw.read::<KeybindState>().enabled.is_some();

        self.uiw.write::<KeybindState>().update(
            &mut self.uiw.write::<Bindings>(),
            &mut self.uiw.write::<InputMap>(),
            &ctx.input,
        );
        self.uiw.write::<InputMap>().prepare_frame(
            &ctx.input,
            !ctx.egui.last_kb_captured,
            !ctx.egui.last_mouse_captured,
        );
        gui::run_ui_systems(&self.sim.read().unwrap(), &self.uiw);

        self.uiw.write::<Timings>().all.add_value(ctx.delta);
        self.uiw.write::<Timings>().per_game_system = self.game_schedule.times();

        self.uiw.write::<GuiState>().hidden ^= self
            .uiw
            .read::<InputMap>()
            .just_act
            .contains(&InputAction::HideInterface);

        manage_settings(ctx, &self.uiw.read::<Settings>());
        self.manage_io(ctx);

        self.map_renderer.update(&self.sim.read().unwrap(), ctx);

        ctx.gfx
            .set_time(self.sim.read().unwrap().read::<GameTime>().timestamp as f32);

        for (sound, kind) in self.uiw.write::<ImmediateSound>().orders.drain(..) {
            ctx.audio.play(sound, kind);
        }
        self.all_audio
            .update(&self.sim.read().unwrap(), &self.uiw, &mut ctx.audio);

        FollowEntity::update_camera(self);
        self.uiw.camera_mut().update(ctx);
        self.manage_gfx_params(ctx);
    }

    fn render(&mut self, ctx: &mut FrameContext<'_>) {
        profiling::scope!("game_loop::render");
        let start = Instant::now();
        let sim = self.sim.read().unwrap();

        let camera = self.uiw.read::<OrbitCamera>();

        let time: GameTime = *self.sim.read().unwrap().read::<GameTime>();

        self.map_renderer.render(
            &sim.map(),
            time.seconds,
            &camera.camera,
            MapRenderOptions {
                show_arrows: self.uiw.read::<Tool>().show_arrows(),
                show_lots: self.uiw.read::<Tool>().show_lots(),
            },
            &mut self.uiw.write::<ImmediateDraw>(),
            ctx,
        );

        self.instanced_renderer
            .render(&self.sim.read().unwrap(), ctx);

        drop(sim);
        drop(camera);

        self.immediate_draw(ctx);

        self.uiw
            .write::<Timings>()
            .render
            .add_value(start.elapsed().as_secs_f32());
    }

    fn resized(&mut self, ctx: &mut Context, size: (u32, u32, f64)) {
        self.uiw
            .write::<OrbitCamera>()
            .resize(ctx, size.0 as f32, size.1 as f32);
    }

    fn exit(&mut self) -> bool {
        if self.uiw.read::<GuiState>().last_save.elapsed() < Duration::from_secs(30) {
            return true;
        }
        let mut estate = self.uiw.write::<ExitState>();
        match *estate {
            ExitState::NoExit => {
                *estate = ExitState::ExitAsk;
            }
            ExitState::ExitAsk => {
                return true;
            }
            ExitState::Saving => {}
        }
        false
    }

    fn render_gui(&mut self, ui: &egui::Context) {
        let sim = self.sim.read().unwrap();
        render_oldgui(ui, &self.uiw, &sim);
    }

    fn render_yakui(&mut self) {
        let sim = self.sim.read().unwrap();
        render_newgui(&self.uiw, &sim);
    }
}

impl State {
    fn reset(&mut self, ctx: &mut Context) {
        ctx.gfx.lamplights.reset(&ctx.gfx.device, &ctx.gfx.queue);
        self.map_renderer = MapRenderer::new(&mut ctx.gfx, &self.sim.read().unwrap());
        self.sim.write().unwrap().map().dispatch_all();
        ctx.gfx.update_simplelit_bg();
    }

    fn manage_gfx_params(&mut self, ctx: &mut Context) {
        let t = std::f32::consts::TAU
            * (ctx.gfx.render_params.value().time - 8.0 * GameTime::HOUR as f32)
            / GameTime::DAY as f32;

        let sun = vec3(t.cos(), t.sin() * 0.5, t.sin() + 0.5).normalize();

        self.uiw.insert(ctx.gfx.perf.as_static());

        let params = ctx.gfx.render_params.value_mut();
        params.time_always = self.uiw.time_always();
        params.sun_col = 4.0
            * sun.z.max(0.0).sqrt().sqrt()
            * LinearColor::new(1.0, 0.95 + sun.z * 0.05, 0.95 + sun.z * 0.05, 1.0);
        let camera = self.uiw.read::<OrbitCamera>();
        params.sun = sun;
        params.viewport = vec2(ctx.gfx.size.0 as f32, ctx.gfx.size.1 as f32);
        params.sun_shadow_proj = camera
            .camera
            .build_sun_shadowmap_matrix(
                sun,
                params.shadow_mapping_resolution as f32,
                &ctx.gfx.frustrum,
            )
            .try_into()
            .unwrap();
        params.unproj_pos = self
            .uiw
            .read::<InputMap>()
            .unprojected
            .unwrap_or_default()
            .xy();
        params.terraforming_mode_radius = matches!(*self.uiw.read::<Tool>(), Tool::Terraforming)
            .then(|| self.uiw.read::<TerraformingResource>().radius)
            .unwrap_or_default();
        drop(camera);
        let c = simulation::colors();
        params.sand_col = c.sand_col.into();
        params.sea_col = c.sea_col.into();
    }

    fn manage_io(&mut self, ctx: &mut Context) {
        let sim = self.sim.read().unwrap();
        let map = sim.map();
        //        self.camera.movespeed = settings.camera_sensibility / 100.0;
        self.uiw.camera_mut().camera_movement(
            ctx,
            ctx.delta,
            &self.uiw.read::<InputMap>(),
            &self.uiw.read::<Settings>(),
            map.environment.bounds().expand(-3000.0),
            |p| map.environment.height(p),
        );
        *self.uiw.write::<Camera>() = self.uiw.read::<OrbitCamera>().camera;

        drop(map);
    }

    fn immediate_draw(&mut self, ctx: &mut FrameContext) {
        profiling::scope!("immediate_draw");

        self.immediate_renderer.clear();

        let mut tess = self.immediate_renderer.mk_tess();

        self.uiw.read::<OrbitCamera>().cull_tess(&mut tess);

        {
            profiling::scope!("debug_objs");
            let sim = self.sim.read().unwrap();
            let objs = self.uiw.read::<DebugObjs>();
            for (val, _, obj) in &objs.0 {
                if *val {
                    obj(&mut tess, &sim, &self.uiw);
                }
            }
        }

        {
            let sim = self.sim.read().unwrap();
            let immediate = &mut *self.uiw.write::<ImmediateDraw>();
            let map = sim.map();
            let mut col = LinearColor::WHITE;
            col.a = 0.2;
            unsafe {
                for v in &*addr_of!(geom::DEBUG_POS) {
                    immediate.circle(*v, 1.0).color(LinearColor::RED);
                }
                for v in &*addr_of!(geom::DEBUG_OBBS) {
                    immediate
                        .obb(*v, map.environment.height(v.center()).unwrap_or(0.0) + 8.0)
                        .color(col);
                }
                for v in &*addr_of!(geom::DEBUG_SPLINES) {
                    immediate
                        .polyline(
                            v.smart_points(1.0, 0.0, 1.0)
                                .map(|x| x.z(10.0))
                                .collect::<Vec<_>>(),
                            5.0,
                            false,
                        )
                        .color(col);
                }
                geom::DEBUG_OBBS.clear();
                geom::DEBUG_SPLINES.clear();
                geom::DEBUG_POS.clear();
            }

            immediate.apply(&mut tess, ctx);
            immediate.orders.clear();
        }

        if let Some(mut x) = self.immediate_renderer.build(ctx.gfx) {
            x.skip_depth = true;
            ctx.draw(x)
        }
    }
}

#[derive(Clone, Default)]
pub struct Timings {
    pub all: History,
    pub world_update: History,
    pub render: History,
    pub engine_render_time: History,
    pub gui_time: History,
    pub total_cpu_time: History,
    pub per_game_system: Vec<(String, f32)>,
}
