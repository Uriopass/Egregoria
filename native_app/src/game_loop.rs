use std::time::Instant;

use winit::dpi::PhysicalSize;
use winit::window::{Fullscreen, Window};

use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use common::{History, Z_LANE};
use egregoria::utils::time::GameTime;
use egregoria::{Egregoria, SerPreparedEgregoria};
use geom::{vec3, Camera, LinearColor, Vec3};
use wgpu_engine::lighting::LightInstance;
use wgpu_engine::{FrameContext, GfxContext, GuiRenderContext, Tesselator};

use crate::audio::GameAudio;
use crate::context::Context;
use crate::gui::inputmap::InputMap;
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
use std::convert::{TryFrom, TryInto};

pub struct State {
    goria: Egregoria,

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
            goria,
            immtess: Tesselator::new(None, 1.0),
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let settings = *self.uiw.read::<Settings>();

        self.uiw.write::<InputMap>().prepare_frame(&ctx.input);
        crate::gui::run_ui_systems(&self.goria, &mut self.uiw);

        let commands = std::mem::take(&mut *self.uiw.write::<WorldCommands>());
        *self.uiw.write::<ReceivedCommands>() = ReceivedCommands::default();

        let mut net_state = self.uiw.write::<NetworkState>();

        let mut inputs_to_apply = None;
        match *net_state {
            NetworkState::Singleplayer(ref mut step) => {
                let goria = &mut self.goria; // mut for tick
                let sched = &mut self.game_schedule;
                let mut timings = self.uiw.write::<Timings>();

                let has_commands = !commands.is_empty();
                let mut commands_once = Some(commands.clone());
                step.prepare_frame(settings.time_warp);
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
                match server.poll(
                    &|| {
                        (
                            SerPreparedEgregoria::try_from(&self.goria)
                                .expect("couldn't serialize world"),
                            Frame(self.goria.get_tick()),
                        )
                    },
                    Some(commands),
                ) {
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
            NetworkState::Client(ref mut client) => match client.poll(commands) {
                PollResult::Wait(commands) => {
                    *self.uiw.write::<WorldCommands>() = commands;
                }
                PollResult::Input(inputs) => {
                    inputs_to_apply = Some(inputs);
                }
                PollResult::GameWorld(commands, prepared_goria) => {
                    if let Ok(x) = prepared_goria.try_into() {
                        self.goria = x;
                    } else {
                        log::error!("couldn't decode serialized goria sent by server");
                        *net_state = NetworkState::Singleplayer(Timestep::default());
                    }
                    *self.uiw.write::<WorldCommands>() = commands;
                }
                PollResult::Disconnect(reason) => {
                    log::error!("got disconnected :-( continuing with server world but it's sad");
                    *net_state = NetworkState::Singleplayer(Timestep::default());
                    self.uiw.write::<NetworkConnectionInfo>().error = reason;
                }
            },
        }

        if let Some(inputs) = inputs_to_apply {
            let mut merged = WorldCommands::default();
            for frame_commands in inputs {
                assert_eq!(frame_commands.frame.0, self.goria.get_tick() + 1);
                let commands: WorldCommands = frame_commands
                    .inputs
                    .iter()
                    .map(|x| x.inp.clone())
                    .collect();
                let t = self.goria.tick(&mut self.game_schedule, &commands);
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

        drop(net_state);

        let real_delta = ctx.delta;
        self.uiw.write::<Timings>().all.add_value(real_delta as f32);

        self.uiw.write::<Timings>().per_game_system = self.game_schedule.times();

        self.gui.hidden ^= ctx.input.keyboard.just_pressed.contains(&KeyCode::H);

        Self::manage_settings(ctx, &settings);
        self.manage_io(ctx);

        self.terrain.update(&mut ctx.gfx, &*self.goria.map());

        let map = self.goria.map();
        //        self.camera.movespeed = settings.camera_sensibility / 100.0;
        self.camera.camera_movement(
            ctx,
            real_delta as f32,
            !self.imgui_render.last_mouse_captured,
            !self.imgui_render.last_kb_captured,
            &settings,
            |p| map.terrain.height(p),
        );
        drop(map);
        *self.uiw.write::<Camera>() = self.camera.camera;

        if !self.imgui_render.last_mouse_captured {
            self.uiw.write::<MouseInfo>().unprojected =
                self.camera.unproject(ctx.input.mouse.screen);
        }

        ctx.gfx
            .set_time(self.goria.read::<GameTime>().timestamp as f32);

        for (sound, kind) in self.uiw.write::<ImmediateSound>().orders.drain(..) {
            ctx.audio.play(sound, kind);
        }
        self.all_audio.update(
            &self.goria,
            &mut self.uiw,
            &mut ctx.audio,
            real_delta as f32,
        );

        self.manage_entity_follow();
        self.camera.update(ctx);
    }

    pub fn render(&mut self, ctx: &mut FrameContext) {
        let start = Instant::now();

        self.terrain.render(&self.uiw, ctx);

        self.immtess.meshbuilder.clear();
        self.camera.cull_tess(&mut self.immtess);

        let time: GameTime = *self.goria.read::<GameTime>();
        self.road_renderer
            .render(&self.goria.map(), time.seconds, &mut self.immtess, ctx);

        self.instanced_renderer.render(&self.goria, ctx);

        {
            let objs = self.uiw.read::<DebugObjs>();
            for (val, _, obj) in &objs.0 {
                if *val {
                    obj(&mut self.immtess, &self.goria, &self.uiw);
                }
            }
        }

        {
            let immediate = &mut *self.uiw.write::<ImmediateDraw>();
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

    pub fn lights(&self) -> (Vec<LightInstance>, LinearColor) {
        let mut lights = vec![];

        let time = self.goria.read::<GameTime>();
        let daysec = time.daysec();

        let map = self.goria.map();
        for x in map.roads().values() {
            let w = x.width * 0.5 - 5.0;
            for (point, dir) in x.points().equipoints_dir(45.0) {
                lights.push(LightInstance {
                    pos: (point + dir.perpendicular() * w).z(Z_LANE).into(),
                    scale: 60.0,
                });
                lights.push(LightInstance {
                    pos: (point - dir.perpendicular() * w).z(Z_LANE).into(),
                    scale: 60.0,
                });
            }
        }

        for i in map.intersections().values() {
            lights.push(LightInstance {
                pos: i.pos.z(Z_LANE).into(),
                scale: 60.0,
            });
        }

        let dark = Vec3::ZERO;
        let bright = vec3(1.0, 1.0, 1.0);

        let col = match time.daytime.hour {
            6..=9 => {
                let c = (daysec / GameTime::HOUR as f64 - 6.0) / 4.0;
                dark.smoothstep(bright, c as f32)
            }
            10..=20 => bright,
            21..=22 => {
                let c = (daysec / GameTime::HOUR as f64 - 21.0) / 1.0;
                bright.smoothstep(dark, c as f32)
            }
            _ => dark,
        };

        (lights, LinearColor::new(col.x, col.y, col.z, 1.0))
    }

    pub fn render_gui(&mut self, window: &Window, ctx: GuiRenderContext) {
        let gui = &mut self.gui;
        let goria = &self.goria;
        let uiworld = &mut self.uiw;

        self.imgui_render.render(ctx, window, gui.hidden, |ui| {
            gui.render(&ui, uiworld, goria);
        });
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
        if !self.uiw.read::<MouseInfo>().just_pressed.is_empty() {
            self.uiw.write::<FollowEntity>().0.take();
        }

        if let Some(e) = self.uiw.read::<FollowEntity>().0 {
            if let Some(pos) = self.goria.pos(e) {
                self.camera.follow(pos.z(0.0));
            }
        }
    }

    fn manage_io(&mut self, ctx: &Context) {
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
    }

    pub fn event(&mut self, window: &Window, event: &winit::event::Event<()>) {
        self.imgui_render.handle_event(window, event);
    }

    pub fn resized(&mut self, ctx: &mut Context, size: PhysicalSize<u32>) {
        self.camera
            .resize(ctx, size.width as f32, size.height as f32);
    }
}

register_resource_noserialize!(Timings);
#[derive(Default)]
pub struct Timings {
    pub all: History,
    pub world_update: History,
    pub render: History,
    pub per_game_system: Vec<(String, f32)>,
}
