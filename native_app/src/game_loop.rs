use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use winit::dpi::PhysicalSize;
use winit::window::{Fullscreen, Window};

use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use common::History;
use egregoria::utils::time::GameTime;
use egregoria::Egregoria;
use geom::{Camera, LinearColor};
use wgpu_engine::{FrameContext, GfxContext, GuiRenderContext, Tesselator};

use crate::audio::GameAudio;
use crate::context::Context;
use crate::gui::windows::debug::DebugObjs;
use crate::gui::windows::settings::Settings;
use crate::gui::{FollowEntity, Gui, Tool, UiTextures};
use crate::inputmap::{Bindings, InputAction, InputMap};
use crate::rendering::egui_wrapper::EguiWrapper;
use crate::rendering::{CameraHandler3D, InstancedRender, MapRenderOptions, MapRenderer};
use crate::uiworld::{SaveLoadState, UiWorld};
use common::saveload::Encoder;
use egregoria::engine_interaction::{WorldCommand, WorldCommands};
use egregoria::utils::scheduler::SeqSchedule;

pub(crate) const VERSION: &str = include_str!("../../VERSION");

/// State is the main struct that contains all the state of the game and game UI.
pub(crate) struct State {
    pub(crate) goria: Arc<RwLock<Egregoria>>,

    pub(crate) uiw: UiWorld,

    pub(crate) game_schedule: SeqSchedule,

    pub(crate) camera: CameraHandler3D,

    egui_render: EguiWrapper,

    instanced_renderer: InstancedRender,
    road_renderer: MapRenderer,
    gui: Gui,
    immtess: Tesselator,

    all_audio: GameAudio,
}

impl State {
    pub(crate) fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler3D::load(ctx.gfx.size);

        let mut egui_render = EguiWrapper::new(&mut ctx.gfx, ctx.el.as_ref().unwrap());
        Gui::set_style(&egui_render.egui);
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

        uiworld.insert(UiTextures::new(&mut egui_render.egui));

        let gui: Gui = common::saveload::JSON::load("gui").unwrap_or_default();
        uiworld.insert(camera.camera);

        log::info!("version is {}", VERSION);

        {
            let s = uiworld.read::<Settings>();
            Self::manage_settings(ctx, &s);
        }

        defer!(log::info!("finished init of game loop"));

        Self {
            uiw: uiworld,
            game_schedule,
            camera,
            egui_render,
            instanced_renderer: InstancedRender::new(&mut ctx.gfx),
            road_renderer: MapRenderer::new(&mut ctx.gfx, &goria),
            gui,
            all_audio: GameAudio::new(&mut ctx.audio),
            goria: Arc::new(RwLock::new(goria)),
            immtess: Tesselator::new(&mut ctx.gfx, None, 1.0),
        }
    }

    #[profiling::function]
    pub(crate) fn update(&mut self, ctx: &mut Context) {
        if self
            .uiw
            .read::<WorldCommands>()
            .iter()
            .any(|x| matches!(x, WorldCommand::ResetSave))
        {
            self.reset();
        }

        crate::network::goria_update(self);

        if !self.egui_render.last_mouse_captured {
            let goria = self.goria.read().unwrap();
            let map = goria.map();
            let unproj = self.camera.unproject(ctx.input.mouse.screen, |p| {
                map.terrain.height(p).map(|x| x + 0.01)
            });

            self.uiw.write::<InputMap>().unprojected = unproj;
        }

        self.uiw.write::<InputMap>().prepare_frame(
            &ctx.input,
            !self.egui_render.last_kb_captured,
            !self.egui_render.last_mouse_captured,
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

        self.road_renderer
            .terrain_update(ctx, &self.goria.read().unwrap());

        ctx.gfx
            .set_time(self.goria.read().unwrap().read::<GameTime>().timestamp as f32);

        for (sound, kind) in self.uiw.write::<ImmediateSound>().orders.drain(..) {
            ctx.audio.play(sound, kind);
        }
        self.all_audio
            .update(&self.goria.read().unwrap(), &mut self.uiw, &mut ctx.audio);

        self.manage_entity_follow();
        self.camera.update(ctx);
    }

    pub(crate) fn reset(&mut self) {
        self.road_renderer.reset();
        self.road_renderer.terrain_dirt_id = 0;
        self.road_renderer.meshb.map_dirt_id = 0;
    }

    #[profiling::function]
    pub(crate) fn render(&mut self, ctx: &mut FrameContext<'_>) {
        let start = Instant::now();
        let goria = self.goria.read().unwrap();

        self.immtess.meshbuilder.clear();
        self.camera.cull_tess(&mut self.immtess);

        let time: GameTime = *self.goria.read().unwrap().read::<GameTime>();

        self.road_renderer.render(
            &goria.map(),
            time.seconds,
            &self.camera.camera,
            &self.camera.frustrum,
            MapRenderOptions {
                show_arrows: self.uiw.read::<Tool>().show_arrows(),
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

    #[profiling::function]
    pub(crate) fn render_gui(&mut self, window: &Window, ctx: GuiRenderContext<'_, '_>) {
        let gui = &mut self.gui;
        let uiworld = &mut self.uiw;
        let pixels_per_point = uiworld.read::<Settings>().gui_scale;

        {
            let goria = self.goria.read().unwrap();
            self.egui_render
                .render(ctx, window, gui.hidden, pixels_per_point, |ui| {
                    gui.render(ui, uiworld, &goria);
                });
        }

        let mut slstate = uiworld.write::<SaveLoadState>();
        if slstate.please_save && !slstate.saving_status.load(Ordering::SeqCst) {
            slstate.please_save = false;
            let cpy = self.goria.clone();
            slstate.saving_status.store(true, Ordering::SeqCst);
            let status = slstate.saving_status.clone();
            std::thread::spawn(move || {
                cpy.read().unwrap().save_to_disk("world");
                status.store(false, Ordering::SeqCst);
            });
        }
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

        ctx.audio.set_settings(settings);
    }

    fn manage_entity_follow(&mut self) {
        if self
            .uiw
            .read::<InputMap>()
            .just_act
            .contains(&InputAction::Close)
        {
            self.uiw.write::<FollowEntity>().0.take();
        }

        if let Some(e) = self.uiw.read::<FollowEntity>().0 {
            if let Some(pos) = self.goria.read().unwrap().pos_any(e) {
                self.camera.follow(pos);
            }
        }
    }

    fn manage_io(&mut self, ctx: &mut Context) {
        let goria = self.goria.read().unwrap();
        let map = goria.map();
        //        self.camera.movespeed = settings.camera_sensibility / 100.0;
        self.camera.camera_movement(
            ctx,
            ctx.delta,
            &self.uiw.read::<InputMap>(),
            &self.uiw.read::<Settings>(),
            |p| map.terrain.height(p),
        );
        *self.uiw.write::<Camera>() = self.camera.camera;

        drop(map);
    }

    pub(crate) fn event(&mut self, event: &winit::event::WindowEvent<'_>) {
        self.egui_render.handle_event(event);
    }

    pub(crate) fn resized(&mut self, ctx: &mut Context, size: PhysicalSize<u32>) {
        self.camera
            .resize(ctx, size.width as f32, size.height as f32);
    }
}

#[derive(Default)]
pub(crate) struct Timings {
    pub(crate) all: History,
    pub(crate) world_update: History,
    pub(crate) render: History,
    pub(crate) per_game_system: Vec<(String, f32)>,
}
