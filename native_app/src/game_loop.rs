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
use crate::gui::inputmap::{InputAction, InputMap};
use crate::gui::windows::debug::DebugObjs;
use crate::gui::windows::network::NetworkConnectionInfo;
use crate::gui::windows::settings::{Settings, ShadowQuality};
use crate::gui::{FollowEntity, Gui, UiTextures};
use crate::input::{KeyCode, KeyboardInfo, MouseInfo};
use crate::network::NetworkState;
use crate::rendering::imgui_wrapper::ImguiWrapper;
use crate::rendering::{CameraHandler3D, InstancedRender, RoadRenderer, TerrainRender};
use crate::uiworld::{ReceivedCommands, UiWorld};
use common::saveload::Encoder;
use common::timestep::Timestep;
use egregoria::engine_interaction::WorldCommands;
use egregoria::utils::scheduler::SeqSchedule;
use networking::{Frame, PollResult, ServerPollResult};

pub struct State {
    goria: Arc<RwLock<Egregoria>>,

    uiw: UiWorld,

    game_schedule: SeqSchedule,

    pub camera: CameraHandler3D,

    imgui_render: ImguiWrapper,

    instanced_renderer: InstancedRender,
    road_renderer: RoadRenderer,
    terrain: TerrainRender,
    gui: Gui,
    immtess: Tesselator,

    all_audio: GameAudio,
}

impl State {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = CameraHandler3D::load(ctx.gfx.size);

        let mut imgui_render = ImguiWrapper::new(&mut ctx.gfx, &ctx.window);
        log::info!("loaded imgui_render");

        let goria: Egregoria =
            Egregoria::load_from_disk("world").unwrap_or_else(|| Egregoria::new(10));
        let game_schedule = Egregoria::schedule();
        let mut uiworld = UiWorld::init();

        uiworld.insert(UiTextures::new(&ctx.gfx, &mut imgui_render.renderer));

        let gui: Gui = common::saveload::JSON::load("gui").unwrap_or_default();
        uiworld.insert(camera.camera);
        uiworld.insert(WorldCommands::default());
        uiworld.insert(InputMap::default());

        log::info!("version is {}", goria_version::VERSION);

        {
            let s = uiworld.read::<Settings>();
            Self::manage_settings(ctx, &s);
        }

        Self {
            uiw: uiworld,
            game_schedule,
            camera,
            imgui_render,
            instanced_renderer: InstancedRender::new(&mut ctx.gfx),
            road_renderer: RoadRenderer::new(&mut ctx.gfx, &goria),
            terrain: TerrainRender::new(&mut ctx.gfx),
            gui,
            all_audio: GameAudio::new(&mut ctx.audio),
            goria: Arc::new(RwLock::new(goria)),
            immtess: Tesselator::new(None, 1.0),
        }
    }

    #[profiling::function]
    pub fn update(&mut self, ctx: &mut Context) {
        if !self.imgui_render.last_mouse_captured {
            let goria = self.goria.read().unwrap();
            let map = goria.map();
            let unproj = self.camera.unproject(ctx.input.mouse.screen, |p| {
                map.terrain.height(p).map(|x| x + 0.01)
            });

            self.uiw.write::<MouseInfo>().unprojected = unproj;
            self.uiw.write::<InputMap>().unprojected = unproj;
        }

        self.uiw.write::<InputMap>().prepare_frame(
            &ctx.input,
            !self.imgui_render.last_kb_captured,
            !self.imgui_render.last_mouse_captured,
        );
        crate::gui::run_ui_systems(&self.goria.read().unwrap(), &mut self.uiw);

        self.goria_update();

        self.uiw.write::<Timings>().all.add_value(ctx.delta as f32);
        self.uiw.write::<Timings>().per_game_system = self.game_schedule.times();

        self.gui.hidden ^= self
            .uiw
            .read::<InputMap>()
            .just_act
            .contains(&InputAction::HideInterface);

        Self::manage_settings(ctx, &*self.uiw.read::<Settings>());
        self.manage_io(ctx);

        self.terrain
            .update(&mut ctx.gfx, &*self.goria.read().unwrap().map());

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

    pub fn goria_update(&mut self) {
        let mut goria = unwrap_orr!(self.goria.try_write(), return); // mut for tick

        let timewarp = self.uiw.read::<Settings>().time_warp;
        let commands = std::mem::take(&mut *self.uiw.write::<WorldCommands>());
        *self.uiw.write::<ReceivedCommands>() = ReceivedCommands::default();

        let mut net_state = self.uiw.write::<NetworkState>();

        let mut inputs_to_apply = None;
        match &mut *net_state {
            NetworkState::Singleplayer(ref mut step) => {
                let sched = &mut self.game_schedule;
                let mut timings = self.uiw.write::<Timings>();

                let has_commands = !commands.is_empty();
                let mut commands_once = Some(commands.clone());
                step.prepare_frame(timewarp);
                while step.tick() || (has_commands && commands_once.is_some()) {
                    let t = goria.tick(sched, &commands_once.take().unwrap_or_default());
                    timings.world_update.add_value(t.as_secs_f32());
                }

                if commands_once.is_none() {
                    *self.uiw.write::<ReceivedCommands>() = ReceivedCommands::new(commands);
                } else {
                    *self.uiw.write::<WorldCommands>() = commands;
                }
            }
            NetworkState::Server(ref mut server) => {
                let polled =
                    server
                        .get_mut()
                        .unwrap()
                        .poll(&goria, Frame(goria.get_tick()), Some(commands));
                match polled {
                    ServerPollResult::Wait(commands) => {
                        if let Some(commands) = commands {
                            *self.uiw.write::<WorldCommands>() = commands;
                        }
                    }
                    ServerPollResult::Input(inputs) => {
                        inputs_to_apply = Some(inputs);
                    }
                }
            }
            NetworkState::Client(ref mut client) => {
                let polled = client.get_mut().unwrap().poll(commands);
                match polled {
                    PollResult::Wait(commands) => {
                        *self.uiw.write::<WorldCommands>() = commands;
                    }
                    PollResult::Input(inputs) => {
                        inputs_to_apply = Some(inputs);
                    }
                    PollResult::GameWorld(commands, prepared_goria) => {
                        *goria = prepared_goria;
                        *self.uiw.write::<WorldCommands>() = commands;
                    }
                    PollResult::Disconnect(reason) => {
                        log::error!(
                            "got disconnected :-( continuing with server world but it's sad"
                        );
                        *net_state = NetworkState::Singleplayer(Timestep::default());
                        self.uiw.write::<NetworkConnectionInfo>().error = reason;
                    }
                }
            }
        }

        if let Some(inputs) = inputs_to_apply {
            let mut merged = WorldCommands::default();
            for frame_commands in inputs {
                assert_eq!(frame_commands.frame.0, goria.get_tick() + 1);
                let commands: WorldCommands = frame_commands
                    .inputs
                    .iter()
                    .map(|x| x.inp.clone())
                    .collect();
                let t = goria.tick(&mut self.game_schedule, &commands);
                self.uiw
                    .write::<Timings>()
                    .world_update
                    .add_value(t.as_secs_f32());
                merged.merge(
                    &frame_commands
                        .inputs
                        .into_iter()
                        .filter(|x| x.sent_by_me)
                        .map(|x| x.inp)
                        .collect::<WorldCommands>(),
                );
            }
            *self.uiw.write::<ReceivedCommands>() = ReceivedCommands::new(merged);
        }
    }

    #[profiling::function]
    pub fn render(&mut self, ctx: &mut FrameContext<'_>) {
        let start = Instant::now();
        let goria = self.goria.read().unwrap();

        self.terrain.draw_terrain(&self.uiw, ctx);

        self.immtess.meshbuilder.clear();
        self.camera.cull_tess(&mut self.immtess);

        let time: GameTime = *self.goria.read().unwrap().read::<GameTime>();
        self.road_renderer
            .render(&goria.map(), time.seconds, &mut self.immtess, ctx);

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
            col.a = 0.1;
            unsafe {
                for v in &geom::DEBUG_OBBS {
                    immediate.obb(*v, 2.0).color(col);
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

        if let Some(mut x) = self.immtess.meshbuilder.build(ctx.gfx, ctx.gfx.palette()) {
            x.translucent = true;
            ctx.draw(x)
        }

        self.uiw
            .write::<Timings>()
            .render
            .add_value(start.elapsed().as_secs_f32());
    }

    #[profiling::function]
    pub fn render_gui(&mut self, window: &Window, ctx: GuiRenderContext<'_, '_>) {
        let gui = &mut self.gui;
        let goria = &self.goria.read().unwrap();
        let uiworld = &mut self.uiw;

        self.imgui_render.render(ctx, window, gui.hidden, |ui| {
            gui.render(ui, uiworld, goria);
        });

        if uiworld.please_save {
            uiworld.please_save = false;
            let cpy = self.goria.clone();
            std::thread::spawn(move || {
                cpy.read().unwrap().save_to_disk("world");
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

        ctx.gfx.set_present_mode(settings.vsync.into());
        ctx.gfx.render_params.value_mut().ssao_enabled = settings.ssao as i32;
        ctx.gfx.render_params.value_mut().realistic_sky = settings.realistic_sky as i32;
        ctx.gfx.render_params.value_mut().shadow_mapping_enabled =
            !matches!(settings.shadows, ShadowQuality::NoShadows) as i32;

        if let Some(v) = match settings.shadows {
            ShadowQuality::Low => Some(512),
            ShadowQuality::Medium => Some(1024),
            ShadowQuality::High => Some(2048),
            ShadowQuality::NoShadows => None,
        } {
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
            .read::<KeyboardInfo>()
            .just_pressed
            .contains(&KeyCode::Escape)
        {
            self.uiw.write::<FollowEntity>().0.take();
        }

        if let Some(e) = self.uiw.read::<FollowEntity>().0 {
            if let Some(pos) = self.goria.read().unwrap().pos(e) {
                self.camera.follow(pos);
            }
        }
    }

    fn manage_io(&mut self, ctx: &mut Context) {
        *self.uiw.write::<KeyboardInfo>() = ctx.input.keyboard.clone();
        *self.uiw.write::<MouseInfo>() = ctx.input.mouse.clone();

        if self.imgui_render.last_kb_captured {
            let kb: &mut KeyboardInfo = &mut self.uiw.write::<KeyboardInfo>();
            kb.just_pressed.clear();
            kb.pressed.clear();
        }

        if self.imgui_render.last_mouse_captured {
            let mouse: &mut MouseInfo = &mut self.uiw.write::<MouseInfo>();
            mouse.just_pressed.clear();
            mouse.pressed.clear();
            mouse.wheel_delta = 0.0;
        }

        let goria = self.goria.read().unwrap();
        let map = goria.map();
        //        self.camera.movespeed = settings.camera_sensibility / 100.0;
        self.camera.camera_movement(
            ctx,
            ctx.delta as f32,
            &*self.uiw.read::<InputMap>(),
            &*self.uiw.read::<Settings>(),
            |p| map.terrain.height(p),
        );
        *self.uiw.write::<Camera>() = self.camera.camera;

        drop(map);
    }

    pub fn event(&mut self, window: &Window, event: &winit::event::Event<'_, ()>) {
        self.imgui_render.handle_event(window, event);
    }

    pub fn resized(&mut self, ctx: &mut Context, size: PhysicalSize<u32>) {
        self.camera
            .resize(ctx, size.width as f32, size.height as f32);
    }
}

#[derive(Default)]
pub struct Timings {
    pub all: History,
    pub world_update: History,
    pub render: History,
    pub per_game_system: Vec<(String, f32)>,
}
