use std::borrow::Cow;
use std::time::Instant;

use winit::dpi::PhysicalSize;
use winit::window::{Fullscreen, Window};

use crate::rendering::immediate::{ImmediateDraw, ImmediateOrder, ImmediateSound, OrderKind};
use common::{GameTime, History};
use egregoria::Egregoria;
use geom::Camera;
use geom::{vec3, LinearColor, Vec2};
use map_model::Map;
use wgpu_engine::lighting::{LightInstance, LightRender};
use wgpu_engine::{FrameContext, GuiRenderContext, SpriteBatch};

use crate::audio::GameAudio;
use crate::context::Context;
use crate::gui::windows::debug::DebugObjs;
use crate::gui::windows::settings::Settings;
use crate::gui::{FollowEntity, Gui, UiTextures};
use crate::input::{KeyboardInfo, MouseInfo};
use crate::network::NetworkState;
use crate::rendering::imgui_wrapper::ImguiWrapper;
use crate::rendering::{
    BackgroundRender, CameraHandler, InstancedRender, MeshRenderer, RoadRenderer,
};
use crate::timestep::Timestep;
use crate::uiworld::{ReceivedCommands, UiWorld};
use common::saveload::Encoder;
use egregoria::engine_interaction::WorldCommands;
use egregoria::utils::scheduler::SeqSchedule;
use networking::PollResult;

pub struct State {
    goria: Egregoria,

    uiw: UiWorld,

    game_schedule: SeqSchedule,

    pub camera: CameraHandler,

    imgui_render: ImguiWrapper,

    instanced_renderer: InstancedRender,
    road_renderer: RoadRenderer,
    bg_renderer: BackgroundRender,
    gui: Gui,
    pub light: LightRender,

    all_audio: GameAudio,
}

impl State {
    pub fn new(ctx: &mut Context) -> Self {
        let camera = common::saveload::JSON::load("camera").map_or_else(
            || {
                CameraHandler::new(
                    ctx.gfx.size.0 as f32,
                    ctx.gfx.size.1 as f32,
                    vec3(0.0, 0.0, 1000.0),
                )
            },
            |camera| CameraHandler {
                camera,
                last_pos: Vec2::ZERO,
                movespeed: 1.0,
            },
        );

        let mut imgui_render = ImguiWrapper::new(&mut ctx.gfx, &ctx.window);

        let goria: Egregoria =
            common::saveload::CompressedBincode::load("world").unwrap_or_else(Egregoria::empty);
        let game_schedule = Egregoria::schedule();

        let mut uiworld = UiWorld::init();

        uiworld.insert(UiTextures::new(&ctx.gfx, &mut imgui_render.renderer));

        let gui: Gui = common::saveload::JSON::load("gui").unwrap_or_default();
        uiworld.insert(camera.camera);
        uiworld.insert(WorldCommands::default());

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
            bg_renderer: BackgroundRender::new(&mut ctx.gfx),
            gui,
            all_audio: GameAudio::new(&mut ctx.audio),
            light: LightRender::new(&mut ctx.gfx),
            goria,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        let settings = *self.uiw.read::<Settings>();

        crate::gui::run_ui_systems(&self.goria, &mut self.uiw);

        if let NetworkState::Server { ref mut server, .. } = *self.uiw.write() {
            server.poll(&self.goria);
        }

        let commands = std::mem::take(&mut *self.uiw.write::<WorldCommands>());

        let mut net_state = self.uiw.write::<NetworkState>();
        match *net_state {
            NetworkState::Singleplayer(ref mut step) => {
                let goria = &mut self.goria; // mut for tick
                let sched = &mut self.game_schedule;
                let mut timings = self.uiw.write::<Timings>();

                let mut commands_once = Some(commands.clone());
                step.go_forward(settings.time_warp, || {
                    let t = goria.tick(
                        Timestep::DT,
                        sched,
                        &commands_once.take().unwrap_or_default(),
                    );
                    timings.world_update.add_value(t.as_secs_f32());
                });

                if commands_once.is_none() {
                    *self.uiw.write::<ReceivedCommands>() = ReceivedCommands::new(commands);
                } else {
                    *self.uiw.write::<WorldCommands>() = commands;
                }
            }
            NetworkState::Client { ref mut client }
            | NetworkState::Server { ref mut client, .. } => match client.poll(commands) {
                PollResult::Wait(commands) => {
                    *self.uiw.write::<WorldCommands>() = commands;
                }
                PollResult::Input(inputs) => {
                    let mut merged = WorldCommands::default();
                    for frame_commands in inputs {
                        let commands: WorldCommands =
                            frame_commands.iter().map(|x| x.inp.clone()).collect();
                        let t = self
                            .goria
                            .tick(Timestep::DT, &mut self.game_schedule, &commands);
                        self.uiw
                            .write::<Timings>()
                            .world_update
                            .add_value(t.as_secs_f32());
                        merged.merge(
                            frame_commands
                                .into_iter()
                                .filter(|x| x.sent_by_me)
                                .map(|x| x.inp)
                                .collect::<WorldCommands>()
                                .iter()
                                .cloned(),
                        );
                    }
                    *self.uiw.write::<ReceivedCommands>() = ReceivedCommands::new(merged);
                }
                PollResult::GameWorld(commands, goria) => {
                    self.goria = goria;
                    *self.uiw.write::<WorldCommands>() = commands;
                }
                PollResult::Error => {
                    log::error!("there was an error polling the client");
                }
                PollResult::Disconnect => {
                    log::error!("got disconnected :-( continuing with server world but it's bad");
                    *net_state = NetworkState::Singleplayer(Timestep::new());
                }
            },
        }

        drop(net_state);

        let real_delta = ctx.delta;
        self.uiw.write::<Timings>().all.add_value(real_delta as f32);

        self.uiw.write::<Timings>().per_game_system = self.game_schedule.times();

        Self::manage_settings(ctx, &settings);
        self.manage_io(ctx);

        self.camera.movespeed = settings.camera_sensibility / 100.0;
        self.camera.camera_movement(
            ctx,
            real_delta as f32,
            !self.imgui_render.last_mouse_captured,
            !self.imgui_render.last_kb_captured,
            &settings,
        );
        *self.uiw.write::<Camera>() = self.camera.camera;

        if !self.imgui_render.last_mouse_captured {
            self.uiw.write::<MouseInfo>().unprojected =
                self.camera.unproject(ctx.input.mouse.screen);
        }

        ctx.gfx
            .set_time(self.goria.read::<GameTime>().timestamp as f32);

        {
            let immediate = self.uiw.read::<ImmediateDraw>();
            for ImmediateOrder { kind, .. } in immediate
                .persistent_orders
                .iter()
                .chain(immediate.orders.iter())
            {
                if let OrderKind::TexturedOBB { ref path, .. } = *kind {
                    ctx.gfx.texture(path, Some("immediate tex"));
                }
            }
        }

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

        self.bg_renderer.draw_background(ctx);

        let mut tess = self.camera.culled_tesselator();

        let time: GameTime = *self.goria.read::<GameTime>();
        self.road_renderer
            .render(&self.goria.read::<Map>(), time.seconds, &mut tess, ctx);

        self.instanced_renderer.render(&self.goria, ctx);

        MeshRenderer::render(&self.goria, &mut tess);

        {
            let objs = self.uiw.read::<DebugObjs>();
            for (val, _, obj) in &objs.0 {
                if *val {
                    obj(&mut tess, &self.goria, &self.uiw);
                }
            }
        }

        {
            let immediate = &mut *self.uiw.write::<ImmediateDraw>();
            for ImmediateOrder { kind, color, z } in immediate
                .persistent_orders
                .iter()
                .chain(immediate.orders.iter())
            {
                let z = *z;
                tess.set_color(*color);
                match *kind {
                    OrderKind::Circle { pos, radius } => {
                        tess.draw_circle(pos, z, radius);
                    }
                    OrderKind::Line {
                        from,
                        to,
                        thickness,
                    } => {
                        tess.draw_stroke(from, to, z, thickness);
                    }
                    OrderKind::StrokeCircle {
                        pos,
                        radius,
                        thickness,
                    } => {
                        tess.draw_stroke_circle(pos, z, radius, thickness);
                    }
                    OrderKind::PolyLine {
                        ref points,
                        thickness,
                    } => {
                        tess.draw_polyline(points, z, thickness);
                    }
                    OrderKind::Polygon { ref poly } => {
                        tess.draw_filled_polygon(poly.as_slice(), z);
                    }
                    OrderKind::OBB(ref obb) => {
                        let [ax1, ax2] = obb.axis();
                        tess.draw_rect_cos_sin(
                            obb.center(),
                            z,
                            ax1.magnitude(),
                            ax2.magnitude(),
                            ax1.normalize(),
                        );
                    }
                    OrderKind::TexturedOBB { obb, ref path } => {
                        let tex = ctx
                            .gfx
                            .read_texture(path)
                            .expect("texture not interned")
                            .clone();
                        ctx.objs.push(Box::new(
                            SpriteBatch::builder(tex)
                                .push(obb.center(), obb.axis()[0], z, *color, (1.0, 1.0))
                                .build(ctx.gfx)
                                .unwrap(),
                        ));
                    }
                }
            }
            immediate.orders.clear();
        }

        if let Some(x) = tess.meshbuilder.build(ctx.gfx) {
            ctx.draw(x)
        }

        self.uiw
            .write::<Timings>()
            .render
            .add_value(start.elapsed().as_secs_f32());
    }

    pub fn lights(&self) -> (Cow<[LightInstance]>, LinearColor) {
        let mut lights = vec![];

        let time = self.goria.read::<GameTime>();
        let daysec = time.daysec();

        let map = self.goria.read::<Map>();
        for x in map.roads().values() {
            let w = x.width * 0.5 - 5.0;
            for (point, dir) in x.generated_points().equipoints_dir(45.0) {
                lights.push(LightInstance {
                    pos: (point + dir.perpendicular() * w).into(),
                    scale: 60.0,
                });
                lights.push(LightInstance {
                    pos: (point - dir.perpendicular() * w).into(),
                    scale: 60.0,
                });
            }
        }

        for i in map.intersections().values() {
            lights.push(LightInstance {
                pos: (i.pos).into(),
                scale: 60.0,
            });
        }

        let dark = vec3(0.1, 0.1, 0.1);
        let bright = vec3(1.0, 1.0, 1.0);

        let col = match time.daytime.hour {
            6..=9 => {
                let c = (daysec / GameTime::HOUR as f64 - 6.0) / 4.0;
                dark.smoothstep(bright, c as f32)
            }
            10..=15 => bright,
            16..=20 => {
                let c = (daysec / GameTime::HOUR as f64 - 16.0) / 5.0;
                bright.smoothstep(dark, c as f32)
            }
            _ => dark,
        };

        (
            Cow::Owned(lights),
            LinearColor::new(col.x, col.y, col.z, 1.0),
        )
    }

    pub fn render_gui(&mut self, window: &Window, ctx: GuiRenderContext) {
        let gui = &mut self.gui;
        let goria = &self.goria;
        let uiworld = &mut self.uiw;

        self.imgui_render.render(ctx, window, |ui| {
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

        ctx.audio.set_settings(settings);
    }

    fn manage_entity_follow(&mut self) {
        if !self.uiw.read::<MouseInfo>().just_pressed.is_empty() {
            self.uiw.write::<FollowEntity>().0.take();
        }

        if let Some(e) = self.uiw.read::<FollowEntity>().0 {
            if let Some(pos) = self.goria.pos(e) {
                self.camera.camera.position.x = pos.x;
                self.camera.camera.position.y = pos.y;
            }
        }
    }

    fn manage_io(&mut self, ctx: &Context) {
        *self.uiw.write::<KeyboardInfo>() = ctx.input.keyboard.clone();
        *self.uiw.write::<MouseInfo>() = ctx.input.mouse.clone();

        if self.imgui_render.last_kb_captured {
            let kb: &mut KeyboardInfo = &mut self.uiw.write::<KeyboardInfo>();
            kb.just_pressed.clear();
            kb.is_pressed.clear();
        }

        if self.imgui_render.last_mouse_captured {
            let mouse: &mut MouseInfo = &mut self.uiw.write::<MouseInfo>();
            mouse.just_pressed.clear();
            mouse.buttons.clear();
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
