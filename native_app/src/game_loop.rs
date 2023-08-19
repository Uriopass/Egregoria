use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use winit::event_loop::ControlFlow;
use winit::window::Fullscreen;

use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use common::History;
use egregoria::utils::time::GameTime;
use egregoria::Egregoria;
use engine::{Context, FrameContext, GfxContext, Tesselator};
use geom::{vec2, vec3, Camera, LinearColor};

use crate::audio::GameAudio;
use crate::gui::windows::debug::DebugObjs;
use crate::gui::windows::settings::Settings;
use crate::gui::{ExitState, FollowEntity, Gui, Tool, UiTextures};
use crate::inputmap::{Bindings, InputAction, InputMap};
use crate::rendering::{CameraHandler3D, InstancedRender, MapRenderOptions, MapRenderer};
use crate::uiworld::{SaveLoadState, UiWorld};
use common::saveload::Encoder;
use egregoria::utils::scheduler::SeqSchedule;

pub const VERSION: &str = include_str!("../../VERSION");

/// State is the main struct that contains all the state of the game and game UI.
pub struct State {
    pub goria: Arc<RwLock<Egregoria>>,
    pub uiw: UiWorld,
    pub game_schedule: SeqSchedule,

    instanced_renderer: InstancedRender,
    map_renderer: MapRenderer,
    gui: Gui,
    immtess: Tesselator<true>,

    all_audio: GameAudio,
}

impl engine::framework::State for State {
    fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler3D::load(ctx.gfx.size);

        Gui::set_style(&ctx.egui.egui);
        log::info!("loaded egui_render");

        let goria: Egregoria =
            Egregoria::load_from_disk("world").unwrap_or_else(|| Egregoria::new(true));
        let game_schedule = Egregoria::schedule();
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

        uiworld.insert(UiTextures::new(&mut ctx.egui.egui));

        let gui: Gui = common::saveload::JSON::load("gui").unwrap_or_default();
        uiworld.insert(camera.camera);
        uiworld.insert(camera);

        log::info!("version is {}", VERSION);

        {
            let s = uiworld.read::<Settings>();
            Self::manage_settings(ctx, &s);
        }

        defer!(log::info!("finished init of game loop"));

        let me = Self {
            uiw: uiworld,
            game_schedule,
            instanced_renderer: InstancedRender::new(&mut ctx.gfx),
            map_renderer: MapRenderer::new(&mut ctx.gfx, &goria),
            gui,
            all_audio: GameAudio::new(&mut ctx.audio),
            goria: Arc::new(RwLock::new(goria)),
            immtess: Tesselator::new(&mut ctx.gfx, None, 1.0),
        };
        me.goria.write().unwrap().map().dispatch_all();
        me
    }

    fn update(&mut self, ctx: &mut Context) {
        profiling::scope!("game_loop::update");

        let mut slstate = self.uiw.write::<SaveLoadState>();
        if slstate.please_save && !slstate.saving_status.load(Ordering::SeqCst) {
            slstate.please_save = false;
            let cpy = self.goria.clone();
            slstate.saving_status.store(true, Ordering::SeqCst);
            let status = slstate.saving_status.clone();
            std::thread::spawn(move || {
                profiling::scope!("game_loop::update::save");
                cpy.read().unwrap().save_to_disk("world");
                status.store(false, Ordering::SeqCst);
            });
        }
        drop(slstate);

        crate::network::goria_update(self);

        if std::mem::take(&mut self.uiw.write::<SaveLoadState>().render_reset) {
            self.reset(ctx);
        }

        if !ctx.egui.last_mouse_captured {
            let goria = self.goria.read().unwrap();
            let map = goria.map();
            let unproj = self
                .uiw
                .read::<CameraHandler3D>()
                .unproject(ctx.input.mouse.screen, |p| {
                    map.terrain.height(p).map(|x| x + 0.01)
                });

            self.uiw.write::<InputMap>().unprojected = unproj;
        }

        self.uiw.write::<InputMap>().prepare_frame(
            &ctx.input,
            !ctx.egui.last_kb_captured,
            !ctx.egui.last_mouse_captured,
        );
        crate::gui::run_ui_systems(&self.goria.read().unwrap(), &mut self.uiw);

        self.uiw.write::<Timings>().all.add_value(ctx.delta);
        self.uiw.write::<Timings>().per_game_system = self.game_schedule.times();

        self.gui.hidden ^= self
            .uiw
            .read::<InputMap>()
            .just_act
            .contains(&InputAction::HideInterface);

        Self::manage_settings(ctx, &self.uiw.read::<Settings>());
        self.manage_io(ctx);

        self.map_renderer.update(&self.goria.read().unwrap(), ctx);

        ctx.gfx
            .set_time(self.goria.read().unwrap().read::<GameTime>().timestamp as f32);

        for (sound, kind) in self.uiw.write::<ImmediateSound>().orders.drain(..) {
            ctx.audio.play(sound, kind);
        }
        self.all_audio
            .update(&self.goria.read().unwrap(), &mut self.uiw, &mut ctx.audio);

        FollowEntity::update_camera(self);
        self.uiw.camera_mut().update(ctx);
        self.manage_gfx_params(ctx);
    }

    fn render(&mut self, ctx: &mut FrameContext<'_>) {
        profiling::scope!("game_loop::render");
        let start = Instant::now();
        let goria = self.goria.read().unwrap();

        self.immtess.meshbuilder.clear();
        let camera = self.uiw.read::<CameraHandler3D>();
        camera.cull_tess(&mut self.immtess);

        let time: GameTime = *self.goria.read().unwrap().read::<GameTime>();

        self.map_renderer.render(
            &goria.map(),
            time.seconds,
            &camera.camera,
            &camera.frustrum,
            MapRenderOptions {
                show_arrows: self.uiw.read::<Tool>().show_arrows(),
                show_lots: self.uiw.read::<Tool>().show_lots(),
            },
            &mut self.uiw.write::<ImmediateDraw>(),
            ctx,
        );

        self.instanced_renderer
            .render(&self.goria.read().unwrap(), ctx);

        {
            let objs = self.uiw.read::<DebugObjs>();
            for (val, _, obj) in &objs.0 {
                if *val {
                    obj(&mut self.immtess, &goria, &self.uiw);
                }
            }
        }

        {
            let immediate = &mut *self.uiw.write::<ImmediateDraw>();

            let mut col = LinearColor::WHITE;
            col.a = 1.0;
            unsafe {
                for v in &geom::DEBUG_OBBS {
                    immediate.obb(*v, 3.0).color(col);
                }
                for v in &geom::DEBUG_SPLINES {
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
            }

            immediate.apply(&mut self.immtess, ctx);
            immediate.orders.clear();
        }

        if let Some(mut x) = self.immtess.meshbuilder.build(ctx.gfx) {
            x.skip_depth = true;
            ctx.draw(x)
        }

        self.uiw
            .write::<Timings>()
            .render
            .add_value(start.elapsed().as_secs_f32());
    }

    fn resized(&mut self, ctx: &mut Context, size: (u32, u32)) {
        self.uiw
            .write::<CameraHandler3D>()
            .resize(ctx, size.0 as f32, size.1 as f32);
    }

    fn exit(&mut self, control_flow: &mut ControlFlow) {
        if self.gui.last_save.elapsed() < Duration::from_secs(30) {
            *control_flow = ControlFlow::Exit;
            return;
        }
        let mut estate = self.uiw.write::<ExitState>();
        match *estate {
            ExitState::NoExit => {
                *estate = ExitState::ExitAsk;
            }
            ExitState::ExitAsk => {
                *control_flow = ControlFlow::Exit;
            }
            ExitState::Saving => {}
        }
    }

    fn render_gui(&mut self, ui: &egui::Context) {
        let goria = self.goria.read().unwrap();
        self.gui.render(ui, &mut self.uiw, &goria);
    }
}

impl State {
    fn reset(&mut self, ctx: &mut Context) {
        ctx.gfx.lamplights.reset(&ctx.gfx.device, &ctx.gfx.queue);
        self.map_renderer = MapRenderer::new(&mut ctx.gfx, &self.goria.read().unwrap());
        self.goria.write().unwrap().map().dispatch_all();
        ctx.gfx.update_simplelit_bg();
    }

    fn manage_gfx_params(&self, ctx: &mut Context) {
        let t = std::f32::consts::TAU
            * (ctx.gfx.render_params.value().time - 8.0 * GameTime::HOUR as f32)
            / GameTime::DAY as f32;

        let sun = vec3(t.cos(), t.sin() * 0.5, t.sin() + 0.5).normalize();

        let params = ctx.gfx.render_params.value_mut();
        params.time_always = (params.time_always + ctx.delta) % 3600.0;
        params.sun_col = 4.0
            * sun.z.max(0.0).sqrt().sqrt()
            * LinearColor::new(1.0, 0.95 + sun.z * 0.05, 0.95 + sun.z * 0.05, 1.0);
        let camera = self.uiw.read::<CameraHandler3D>();
        params.cam_pos = camera.camera.eye();
        params.cam_dir = -camera.camera.dir();
        params.sun = sun;
        params.viewport = vec2(ctx.gfx.size.0 as f32, ctx.gfx.size.1 as f32);
        params.sun_shadow_proj = camera
            .camera
            .build_sun_shadowmap_matrix(
                sun,
                params.shadow_mapping_resolution as f32,
                &camera.frustrum,
            )
            .try_into()
            .unwrap();
        drop(camera);
        let c = egregoria::config();
        params.grass_col = c.grass_col.into();
        params.sand_col = c.sand_col.into();
        params.sea_col = c.sea_col.into();
        drop(c);
    }

    fn manage_settings(ctx: &mut Context, settings: &Settings) {
        if settings.fullscreen && ctx.window.fullscreen().is_none() {
            ctx.window
                .set_fullscreen(Some(Fullscreen::Borderless(ctx.window.current_monitor())))
        }
        if !settings.fullscreen && ctx.window.fullscreen().is_some() {
            ctx.window.set_fullscreen(None);
        }

        ctx.gfx.set_vsync(settings.vsync);
        let params = ctx.gfx.render_params.value_mut();
        params.ssao_enabled = settings.ssao as i32;
        params.grid_enabled = settings.terrain_grid as i32;
        params.shadow_mapping_resolution = settings.shadows.size().unwrap_or(0) as i32;

        if let Some(v) = settings.shadows.size() {
            if ctx.gfx.sun_shadowmap.extent.width != v {
                ctx.gfx.sun_shadowmap = GfxContext::mk_shadowmap(&ctx.gfx.device, v);
                ctx.gfx.update_simplelit_bg();
            }
        }

        ctx.egui.pixels_per_point = settings.gui_scale;

        ctx.audio.set_settings(
            settings.master_volume_percent,
            settings.ui_volume_percent,
            settings.music_volume_percent,
            settings.effects_volume_percent,
        );
    }

    fn manage_io(&mut self, ctx: &mut Context) {
        let goria = self.goria.read().unwrap();
        let map = goria.map();
        //        self.camera.movespeed = settings.camera_sensibility / 100.0;
        self.uiw.camera_mut().camera_movement(
            ctx,
            ctx.delta,
            &self.uiw.read::<InputMap>(),
            &self.uiw.read::<Settings>(),
            |p| map.terrain.height(p),
        );
        *self.uiw.write::<Camera>() = self.uiw.read::<CameraHandler3D>().camera;

        drop(map);
    }
}

#[derive(Default)]
pub struct Timings {
    pub all: History,
    pub world_update: History,
    pub render: History,
    pub per_game_system: Vec<(String, f32)>,
}
