use crate::gui::imgui_wrapper::ImGuiWrapper;
use crate::rendering::camera_handler::CameraHandler;
use crate::rendering::instanced_render::InstancedRender;
use crate::rendering::render_context::RenderContext;
use crate::rendering::road_rendering::RoadRenderer;
use crate::rendering::sorted_mesh_renderer::SortedMeshRenderer;
use cgmath::Vector2;
use ggez::graphics::{Color, DrawMode, DrawParam, Font};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::input::mouse::MouseButton;
use ggez::{filesystem, graphics, timer, Context, GameResult};
use scale::engine_interaction;
use scale::engine_interaction::{KeyboardInfo, MouseInfo, RenderStats, TimeInfo};
use scale::geometry::intersections::intersection_point;
use scale::gui::Gui;
use scale::interaction::FollowEntity;
use scale::map_model::{Map, MapUIState, TraverseKind};
use scale::pedestrians::PedestrianComponent;
use scale::physics::{CollisionWorld, Transform};
use scale::specs::Join;
use scale::specs::{Dispatcher, RunNow, World, WorldExt};
use std::collections::HashSet;
use std::iter::FromIterator;

pub struct EngineState<'a> {
    pub world: World,
    pub dispatch: Dispatcher<'a, 'a>,
    pub cam: CameraHandler,
    pub render_enabled: bool,
    pub grid: bool,
    pub font: Option<Font>,
    pub imgui_wrapper: ImGuiWrapper,
    pub sorted_mesh_render: SortedMeshRenderer,
    pub road_render: RoadRenderer,
    pub instanced_render: InstancedRender,
    pub time_sync: f64,
}

impl<'a> EngineState<'a> {
    pub(crate) fn new(
        world: World,
        dispatch: Dispatcher<'a, 'a>,
        mut ctx: &mut Context,
    ) -> GameResult<EngineState<'a>> {
        println!("{}", filesystem::resources_dir(ctx).display());

        let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf").ok();
        //        let text = graphics::Text::new(("Hello world!", font, 48.0));
        //       let test: Image = graphics::Image::new(ctx, "/test.png")?;

        graphics::set_resizable(ctx, true)?;
        let (width, height) = graphics::drawable_size(ctx);
        let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
        Ok(EngineState {
            font,
            world,
            dispatch,
            cam: CameraHandler::new(width, height),
            render_enabled: true,
            grid: true,
            imgui_wrapper,
            sorted_mesh_render: SortedMeshRenderer::new(),
            road_render: RoadRenderer::new(),
            instanced_render: InstancedRender::new(ctx),
            time_sync: 0.0,
        })
    }
}

impl EngineState<'_> {
    fn tick(&mut self, ctx: &mut Context) {
        let start_update = std::time::Instant::now();
        let pressed: Vec<engine_interaction::MouseButton> =
            if !self.imgui_wrapper.last_mouse_captured {
                vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                    .into_iter()
                    .filter(|x| ggez::input::mouse::button_pressed(ctx, *x))
                    .map(scale_mb)
                    .collect()
            } else {
                vec![]
            };
        if self.imgui_wrapper.last_kb_captured {
            self.world
                .write_resource::<KeyboardInfo>()
                .just_pressed
                .clear();
        }

        self.imgui_wrapper.update_mouse_down((
            ggez::input::mouse::button_pressed(ctx, MouseButton::Left),
            ggez::input::mouse::button_pressed(ctx, MouseButton::Right),
            ggez::input::mouse::button_pressed(ctx, MouseButton::Middle),
        ));

        // info from last frame to determine "just pressed"
        let last_pressed = self.world.read_resource::<MouseInfo>().buttons.clone();

        *self.world.write_resource::<MouseInfo>() = MouseInfo {
            unprojected: self.cam.unproject_mouse_click(ctx),
            buttons: HashSet::from_iter(pressed.clone()),
            just_pressed: HashSet::from_iter(
                pressed.into_iter().filter(|x| !last_pressed.contains(x)),
            ),
        };

        self.dispatch.run_now(&self.world);
        self.world.maintain();

        self.cam.easy_camera_movement(
            ctx,
            timer::delta(ctx).as_secs_f32(),
            !self.imgui_wrapper.last_mouse_captured,
            !self.imgui_wrapper.last_kb_captured,
        );

        if !self
            .world
            .read_resource::<MouseInfo>()
            .just_pressed
            .is_empty()
        {
            self.world.write_resource::<FollowEntity>().0.take();
        }

        if let Some(e) = self.world.read_resource::<FollowEntity>().0 {
            if let Some(pos) = self
                .world
                .read_component::<Transform>()
                .get(e)
                .map(|x| x.position())
            {
                self.cam.camera.position = pos;
            }
        }
        self.cam.update(ctx);

        self.world
            .write_resource::<KeyboardInfo>()
            .just_pressed
            .clear();
        self.world.write_resource::<RenderStats>().update_time =
            (std::time::Instant::now() - start_update).as_secs_f32();
    }
}

const TIME_STEP: f64 = 1.0 / 30.0;

impl<'a> ggez::event::EventHandler for EngineState<'a> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f64();

        let time = self.world.read_resource::<TimeInfo>();
        self.time_sync += delta * time.time_speed;
        let mut ticks_to_do = (((self.time_sync - time.time) / TIME_STEP) as u32).max(0);

        if ticks_to_do > 1 {
            ticks_to_do = 1;
            self.time_sync = time.time + 1.0 * TIME_STEP;
        }
        drop(time);

        for _ in 0..ticks_to_do {
            let mut time = self.world.write_resource::<TimeInfo>();
            time.delta = TIME_STEP as f32;
            time.time += TIME_STEP;
            time.time_seconds = time.time as u64;
            drop(time);

            self.tick(ctx);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let start_draw = std::time::Instant::now();

        let time: TimeInfo = *self.world.read_resource::<TimeInfo>();

        let mut rc = RenderContext::new(&mut self.cam, ctx, self.font);
        rc.clear();

        // Render grid
        if self.grid && rc.cam.camera.zoom > 3.0 {
            let gray_maj = (rc.cam.camera.zoom / 40.0).min(0.2);
            let gray_min = gray_maj / 2.0;
            if rc.cam.camera.zoom > 6.0 {
                rc.draw_grid(1.0, Color::new(gray_min, gray_min, gray_min, 1.0));
            }
            rc.draw_grid(10.0, Color::new(gray_maj, gray_maj, gray_maj, 1.0));
            rc.flush()?;
        }

        {
            if self.render_enabled {
                if self.world.read_resource::<MapUIState>().map_render_dirty
                    || self.road_render.mesh.is_none()
                {
                    self.road_render.build_mesh(
                        &self.world.read_resource::<Map>(),
                        time.time_seconds,
                        &mut rc,
                    );
                }
                if let Some(m) = &self.road_render.mesh {
                    ggez::graphics::draw(rc.ctx, m, DrawParam::default())?;
                }

                self.sorted_mesh_render.render(&mut self.world, &mut rc);
                self.instanced_render.render(&mut self.world, &mut rc);
            }
        }

        rc.finish()?;

        let mut gui: Gui = (*self.world.read_resource::<Gui>()).clone();
        self.imgui_wrapper
            .render(ctx, &mut self.world, &mut gui, 1.0);
        *self.world.write_resource::<Gui>() = gui;

        self.world.write_resource::<RenderStats>().render_time =
            (std::time::Instant::now() - start_draw).as_secs_f32();

        graphics::present(ctx)
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        if !self.imgui_wrapper.last_mouse_captured {
            if y > 0.0 {
                self.cam.easy_camera_movement_keys(ctx, KeyCode::Add);
            }
            if y < 0.0 {
                self.cam.easy_camera_movement_keys(ctx, KeyCode::Subtract);
            }
        }
        self.imgui_wrapper.update_wheel(y);
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        self.world
            .write_resource::<KeyboardInfo>()
            .just_pressed
            .insert(scale_kc(keycode));
        if keycode == KeyCode::R {
            self.render_enabled = !self.render_enabled;
        }
        if keycode == KeyCode::G {
            self.grid = !self.grid;
        }
        //println!("Key pressed {:?}", keycode);

        match keycode {
            KeyCode::Delete => self.imgui_wrapper.delete(),
            KeyCode::Back => self.imgui_wrapper.backspace(),
            KeyCode::Return => self.imgui_wrapper.enter(),
            KeyCode::Left => self.imgui_wrapper.left_arrow(),
            KeyCode::Right => self.imgui_wrapper.right_arrow(),
            KeyCode::Tab => self.imgui_wrapper.tab(),
            KeyCode::Key0 => self.imgui_wrapper.queue_char('0'),
            KeyCode::Key1 => self.imgui_wrapper.queue_char('1'),
            KeyCode::Key2 => self.imgui_wrapper.queue_char('2'),
            KeyCode::Key3 => self.imgui_wrapper.queue_char('3'),
            KeyCode::Key4 => self.imgui_wrapper.queue_char('4'),
            KeyCode::Key5 => self.imgui_wrapper.queue_char('5'),
            KeyCode::Key6 => self.imgui_wrapper.queue_char('6'),
            KeyCode::Key7 => self.imgui_wrapper.queue_char('7'),
            KeyCode::Key8 => self.imgui_wrapper.queue_char('8'),
            KeyCode::Key9 => self.imgui_wrapper.queue_char('9'),
            KeyCode::Numpad0 => self.imgui_wrapper.queue_char('0'),
            KeyCode::Numpad1 => self.imgui_wrapper.queue_char('1'),
            KeyCode::Numpad2 => self.imgui_wrapper.queue_char('2'),
            KeyCode::Numpad3 => self.imgui_wrapper.queue_char('3'),
            KeyCode::Numpad4 => self.imgui_wrapper.queue_char('4'),
            KeyCode::Numpad5 => self.imgui_wrapper.queue_char('5'),
            KeyCode::Numpad6 => self.imgui_wrapper.queue_char('6'),
            KeyCode::Numpad7 => self.imgui_wrapper.queue_char('7'),
            KeyCode::Numpad8 => self.imgui_wrapper.queue_char('8'),
            KeyCode::Numpad9 => self.imgui_wrapper.queue_char('9'),
            KeyCode::Period => self.imgui_wrapper.queue_char('.'),
            KeyCode::Minus => self.imgui_wrapper.queue_char('-'),
            _ => (),
        }
        if !self.imgui_wrapper.last_kb_captured {
            self.cam.easy_camera_movement_keys(ctx, keycode);
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.cam.resize(ctx, width, height);
    }
}

fn scale_mb(x: MouseButton) -> scale::engine_interaction::MouseButton {
    match x {
        MouseButton::Left => scale::engine_interaction::MouseButton::Left,
        MouseButton::Right => scale::engine_interaction::MouseButton::Right,
        MouseButton::Middle => scale::engine_interaction::MouseButton::Middle,
        MouseButton::Other(x) => scale::engine_interaction::MouseButton::Other(x),
    }
}

/*
use gfx::*;

// Define the input struct for our shader.
gfx_defines! {
    constant Dim {
        rate: f32 = "u_Rate",
    }
}

fn shader_test(sself: &mut EngineState, ctx: &mut Context) -> GameResult<()> {
    let img = Image::new(ctx, "/test.png").ok();
    let mut sr = Tesselator::new(&sself.cam.get_screen_box(), sself.cam.camera.zoom, img);
    sr.draw_rect_cos_sin(vec2(0.0, 0.0), 100.0, 100.0, vec2(0.0, 1.0));
    let mesh: Mesh = sr.meshbuilder.build(ctx)?;
    let dim = Dim { rate: 0.5 };
    let shader = ggez::graphics::Shader::new(ctx, "/test.glslv", "/test.glslf", dim, "Dim", None)?;
    let l = ggez::graphics::use_shader(ctx, &shader);
    ggez::graphics::draw(ctx, &mesh, DrawParam::new())?;
    Ok(())
}*/

#[allow(dead_code)]
fn debug_coworld(rc: &mut RenderContext, world: &World) -> GameResult<()> {
    let lol = world.read_resource::<CollisionWorld>();
    rc.draw_grid(50.0, Color::new(0.0, 0.0, 1.0, 1.0));
    rc.flush()?;
    rc.tess.mode = DrawMode::stroke(0.1);
    rc.tess.color = Color::new(0.8, 0.8, 0.9, 0.5);
    for x in lol.cells() {
        for y in &x.objs {
            rc.tess.draw_circle(y.pos, 10.0);
            rc.draw_text(
                &format!("{}", lol.query_around(y.pos, 10.0).count()),
                y.pos,
                5.0,
                Color::new(1.0, 1.0, 1.0, 1.0),
            )?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn debug_rays(rc: &mut RenderContext, time: TimeInfo) {
    let c = time.time.cos() as f32;
    let s = time.time.sin() as f32;

    let r = scale::geometry::intersections::Ray {
        from: 10.0 * Vector2::new(c, s),
        dir: Vector2::new(
            (time.time * 2.3 + 1.0).cos() as f32,
            (time.time * 2.3 + 1.0).sin() as f32,
        ),
    };

    let r2 = scale::geometry::intersections::Ray {
        from: 10.0 * Vector2::new((time.time as f32 * 1.5 + 3.0).cos(), s * 2.0),
        dir: Vector2::new(c, -s),
    };

    let inter = intersection_point(r, r2);

    rc.tess.color = ggez::graphics::WHITE;
    rc.tess.draw_line(r.from, r.from + r.dir * 50.0);
    rc.tess.draw_line(r2.from, r2.from + r2.dir * 50.0);

    if let Some(v) = inter {
        rc.tess.color.r = 1.0;
        rc.tess.color.g = 0.0;
        rc.tess.color.b = 0.0;

        rc.tess.draw_circle(v, 2.0);
    }
}

// Thanks multi cursor
fn scale_kc(x: KeyCode) -> scale::engine_interaction::KeyCode {
    match x {
        KeyCode::Key1 => scale::engine_interaction::KeyCode::Key1,
        KeyCode::Key2 => scale::engine_interaction::KeyCode::Key2,
        KeyCode::Key3 => scale::engine_interaction::KeyCode::Key3,
        KeyCode::Key4 => scale::engine_interaction::KeyCode::Key4,
        KeyCode::Key5 => scale::engine_interaction::KeyCode::Key5,
        KeyCode::Key6 => scale::engine_interaction::KeyCode::Key6,
        KeyCode::Key7 => scale::engine_interaction::KeyCode::Key7,
        KeyCode::Key8 => scale::engine_interaction::KeyCode::Key8,
        KeyCode::Key9 => scale::engine_interaction::KeyCode::Key9,
        KeyCode::Key0 => scale::engine_interaction::KeyCode::Key0,
        KeyCode::A => scale::engine_interaction::KeyCode::A,
        KeyCode::B => scale::engine_interaction::KeyCode::B,
        KeyCode::C => scale::engine_interaction::KeyCode::C,
        KeyCode::D => scale::engine_interaction::KeyCode::D,
        KeyCode::E => scale::engine_interaction::KeyCode::E,
        KeyCode::F => scale::engine_interaction::KeyCode::F,
        KeyCode::G => scale::engine_interaction::KeyCode::G,
        KeyCode::H => scale::engine_interaction::KeyCode::H,
        KeyCode::I => scale::engine_interaction::KeyCode::I,
        KeyCode::J => scale::engine_interaction::KeyCode::J,
        KeyCode::K => scale::engine_interaction::KeyCode::K,
        KeyCode::L => scale::engine_interaction::KeyCode::L,
        KeyCode::M => scale::engine_interaction::KeyCode::M,
        KeyCode::N => scale::engine_interaction::KeyCode::N,
        KeyCode::O => scale::engine_interaction::KeyCode::O,
        KeyCode::P => scale::engine_interaction::KeyCode::P,
        KeyCode::Q => scale::engine_interaction::KeyCode::Q,
        KeyCode::R => scale::engine_interaction::KeyCode::R,
        KeyCode::S => scale::engine_interaction::KeyCode::S,
        KeyCode::T => scale::engine_interaction::KeyCode::T,
        KeyCode::U => scale::engine_interaction::KeyCode::U,
        KeyCode::V => scale::engine_interaction::KeyCode::V,
        KeyCode::W => scale::engine_interaction::KeyCode::W,
        KeyCode::X => scale::engine_interaction::KeyCode::X,
        KeyCode::Y => scale::engine_interaction::KeyCode::Y,
        KeyCode::Z => scale::engine_interaction::KeyCode::Z,
        KeyCode::Escape => scale::engine_interaction::KeyCode::Escape,
        KeyCode::F1 => scale::engine_interaction::KeyCode::F1,
        KeyCode::F2 => scale::engine_interaction::KeyCode::F2,
        KeyCode::F3 => scale::engine_interaction::KeyCode::F3,
        KeyCode::F4 => scale::engine_interaction::KeyCode::F4,
        KeyCode::F5 => scale::engine_interaction::KeyCode::F5,
        KeyCode::F6 => scale::engine_interaction::KeyCode::F6,
        KeyCode::F7 => scale::engine_interaction::KeyCode::F7,
        KeyCode::F8 => scale::engine_interaction::KeyCode::F8,
        KeyCode::F9 => scale::engine_interaction::KeyCode::F9,
        KeyCode::F10 => scale::engine_interaction::KeyCode::F10,
        KeyCode::F11 => scale::engine_interaction::KeyCode::F11,
        KeyCode::F12 => scale::engine_interaction::KeyCode::F12,
        KeyCode::F13 => scale::engine_interaction::KeyCode::F13,
        KeyCode::F14 => scale::engine_interaction::KeyCode::F14,
        KeyCode::F15 => scale::engine_interaction::KeyCode::F15,
        KeyCode::F16 => scale::engine_interaction::KeyCode::F16,
        KeyCode::F17 => scale::engine_interaction::KeyCode::F17,
        KeyCode::F18 => scale::engine_interaction::KeyCode::F18,
        KeyCode::F19 => scale::engine_interaction::KeyCode::F19,
        KeyCode::F20 => scale::engine_interaction::KeyCode::F20,
        KeyCode::F21 => scale::engine_interaction::KeyCode::F21,
        KeyCode::F22 => scale::engine_interaction::KeyCode::F22,
        KeyCode::F23 => scale::engine_interaction::KeyCode::F23,
        KeyCode::F24 => scale::engine_interaction::KeyCode::F24,
        KeyCode::Snapshot => scale::engine_interaction::KeyCode::Snapshot,
        KeyCode::Scroll => scale::engine_interaction::KeyCode::Scroll,
        KeyCode::Pause => scale::engine_interaction::KeyCode::Pause,
        KeyCode::Insert => scale::engine_interaction::KeyCode::Insert,
        KeyCode::Home => scale::engine_interaction::KeyCode::Home,
        KeyCode::Delete => scale::engine_interaction::KeyCode::Delete,
        KeyCode::End => scale::engine_interaction::KeyCode::End,
        KeyCode::PageDown => scale::engine_interaction::KeyCode::PageDown,
        KeyCode::PageUp => scale::engine_interaction::KeyCode::PageUp,
        KeyCode::Left => scale::engine_interaction::KeyCode::Left,
        KeyCode::Up => scale::engine_interaction::KeyCode::Up,
        KeyCode::Right => scale::engine_interaction::KeyCode::Right,
        KeyCode::Down => scale::engine_interaction::KeyCode::Down,
        KeyCode::Back => scale::engine_interaction::KeyCode::Backspace,
        KeyCode::Return => scale::engine_interaction::KeyCode::Return,
        KeyCode::Space => scale::engine_interaction::KeyCode::Space,
        KeyCode::Compose => scale::engine_interaction::KeyCode::Compose,
        KeyCode::Caret => scale::engine_interaction::KeyCode::Caret,
        KeyCode::Numlock => scale::engine_interaction::KeyCode::Numlock,
        KeyCode::Numpad0 => scale::engine_interaction::KeyCode::Numpad0,
        KeyCode::Numpad1 => scale::engine_interaction::KeyCode::Numpad1,
        KeyCode::Numpad2 => scale::engine_interaction::KeyCode::Numpad2,
        KeyCode::Numpad3 => scale::engine_interaction::KeyCode::Numpad3,
        KeyCode::Numpad4 => scale::engine_interaction::KeyCode::Numpad4,
        KeyCode::Numpad5 => scale::engine_interaction::KeyCode::Numpad5,
        KeyCode::Numpad6 => scale::engine_interaction::KeyCode::Numpad6,
        KeyCode::Numpad7 => scale::engine_interaction::KeyCode::Numpad7,
        KeyCode::Numpad8 => scale::engine_interaction::KeyCode::Numpad8,
        KeyCode::Numpad9 => scale::engine_interaction::KeyCode::Numpad9,
        KeyCode::AbntC1 => scale::engine_interaction::KeyCode::AbntC1,
        KeyCode::AbntC2 => scale::engine_interaction::KeyCode::AbntC2,
        KeyCode::Add => scale::engine_interaction::KeyCode::Add,
        KeyCode::Apostrophe => scale::engine_interaction::KeyCode::Apostrophe,
        KeyCode::Apps => scale::engine_interaction::KeyCode::Apps,
        KeyCode::At => scale::engine_interaction::KeyCode::At,
        KeyCode::Ax => scale::engine_interaction::KeyCode::Ax,
        KeyCode::Backslash => scale::engine_interaction::KeyCode::Backslash,
        KeyCode::Calculator => scale::engine_interaction::KeyCode::Calculator,
        KeyCode::Capital => scale::engine_interaction::KeyCode::Capital,
        KeyCode::Colon => scale::engine_interaction::KeyCode::Colon,
        KeyCode::Comma => scale::engine_interaction::KeyCode::Comma,
        KeyCode::Convert => scale::engine_interaction::KeyCode::Convert,
        KeyCode::Decimal => scale::engine_interaction::KeyCode::Decimal,
        KeyCode::Divide => scale::engine_interaction::KeyCode::Divide,
        KeyCode::Equals => scale::engine_interaction::KeyCode::Equals,
        KeyCode::Grave => scale::engine_interaction::KeyCode::Grave,
        KeyCode::Kana => scale::engine_interaction::KeyCode::Kana,
        KeyCode::Kanji => scale::engine_interaction::KeyCode::Kanji,
        KeyCode::LAlt => scale::engine_interaction::KeyCode::LAlt,
        KeyCode::LBracket => scale::engine_interaction::KeyCode::LBracket,
        KeyCode::LControl => scale::engine_interaction::KeyCode::LControl,
        KeyCode::LShift => scale::engine_interaction::KeyCode::LShift,
        KeyCode::LWin => scale::engine_interaction::KeyCode::LWin,
        KeyCode::Mail => scale::engine_interaction::KeyCode::Mail,
        KeyCode::MediaSelect => scale::engine_interaction::KeyCode::MediaSelect,
        KeyCode::MediaStop => scale::engine_interaction::KeyCode::MediaStop,
        KeyCode::Minus => scale::engine_interaction::KeyCode::Minus,
        KeyCode::Multiply => scale::engine_interaction::KeyCode::Multiply,
        KeyCode::Mute => scale::engine_interaction::KeyCode::Mute,
        KeyCode::MyComputer => scale::engine_interaction::KeyCode::MyComputer,
        KeyCode::NavigateForward => scale::engine_interaction::KeyCode::NavigateForward,
        KeyCode::NavigateBackward => scale::engine_interaction::KeyCode::NavigateBackward,
        KeyCode::NextTrack => scale::engine_interaction::KeyCode::NextTrack,
        KeyCode::NoConvert => scale::engine_interaction::KeyCode::NoConvert,
        KeyCode::NumpadComma => scale::engine_interaction::KeyCode::NumpadComma,
        KeyCode::NumpadEnter => scale::engine_interaction::KeyCode::NumpadEnter,
        KeyCode::NumpadEquals => scale::engine_interaction::KeyCode::NumpadEquals,
        KeyCode::OEM102 => scale::engine_interaction::KeyCode::OEM102,
        KeyCode::Period => scale::engine_interaction::KeyCode::Period,
        KeyCode::PlayPause => scale::engine_interaction::KeyCode::PlayPause,
        KeyCode::Power => scale::engine_interaction::KeyCode::Power,
        KeyCode::PrevTrack => scale::engine_interaction::KeyCode::PrevTrack,
        KeyCode::RAlt => scale::engine_interaction::KeyCode::RAlt,
        KeyCode::RBracket => scale::engine_interaction::KeyCode::RBracket,
        KeyCode::RControl => scale::engine_interaction::KeyCode::RControl,
        KeyCode::RShift => scale::engine_interaction::KeyCode::RShift,
        KeyCode::RWin => scale::engine_interaction::KeyCode::RWin,
        KeyCode::Semicolon => scale::engine_interaction::KeyCode::Semicolon,
        KeyCode::Slash => scale::engine_interaction::KeyCode::Slash,
        KeyCode::Sleep => scale::engine_interaction::KeyCode::Sleep,
        KeyCode::Stop => scale::engine_interaction::KeyCode::Stop,
        KeyCode::Subtract => scale::engine_interaction::KeyCode::Subtract,
        KeyCode::Sysrq => scale::engine_interaction::KeyCode::Sysrq,
        KeyCode::Tab => scale::engine_interaction::KeyCode::Tab,
        KeyCode::Underline => scale::engine_interaction::KeyCode::Underline,
        KeyCode::Unlabeled => scale::engine_interaction::KeyCode::Unlabeled,
        KeyCode::VolumeDown => scale::engine_interaction::KeyCode::VolumeDown,
        KeyCode::VolumeUp => scale::engine_interaction::KeyCode::VolumeUp,
        KeyCode::Wake => scale::engine_interaction::KeyCode::Wake,
        KeyCode::WebBack => scale::engine_interaction::KeyCode::WebBack,
        KeyCode::WebFavorites => scale::engine_interaction::KeyCode::WebFavorites,
        KeyCode::WebForward => scale::engine_interaction::KeyCode::WebForward,
        KeyCode::WebHome => scale::engine_interaction::KeyCode::WebHome,
        KeyCode::WebRefresh => scale::engine_interaction::KeyCode::WebRefresh,
        KeyCode::WebSearch => scale::engine_interaction::KeyCode::WebSearch,
        KeyCode::WebStop => scale::engine_interaction::KeyCode::WebStop,
        KeyCode::Yen => scale::engine_interaction::KeyCode::Yen,
        KeyCode::Copy => scale::engine_interaction::KeyCode::Copy,
        KeyCode::Paste => scale::engine_interaction::KeyCode::Paste,
        KeyCode::Cut => scale::engine_interaction::KeyCode::Cut,
    }
}
