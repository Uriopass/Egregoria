use std::env;
use std::path;

use ggez::conf::NumSamples;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::*;

use crate::engine::camera_handler::CameraHandler;
use crate::engine::render_context::RenderContext;

pub mod camera;
pub mod camera_handler;
pub mod drawable;
pub mod render_context;
pub mod shape_render;

struct EngineState {
    pub time: f32,
    pub cam: CameraHandler,
}

impl EngineState {
    fn new(ctx: &mut Context) -> GameResult<EngineState> {
        println!("{}", filesystem::resources_dir(ctx).display());

        //let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        //let text = graphics::Text::new(("Hello world!", font, 48.0));
        //let test: Image = graphics::Image::new(ctx, "/test.png")?;

        graphics::set_resizable(ctx, true)?;
        Ok(EngineState {
            time: 0.,
            cam: CameraHandler::new(),
        })
    }
}

impl ggez::event::EventHandler for EngineState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        self.time += delta;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.cam.easy_camera_movement(ctx);
        self.cam.update(ctx);

        let mut rc = RenderContext::new(&mut self.cam, ctx);

        rc.clear();

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx)?;
        graphics::present(ctx)
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        if y > 0. {
            self.cam.easy_camera_movement_keys(ctx, KeyCode::Add);
        }
        if y < 0. {
            self.cam.easy_camera_movement_keys(ctx, KeyCode::Subtract);
        }
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        self.cam.easy_camera_movement_keys(ctx, keycode);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.cam.resize(ctx, width, height);
    }
}

pub fn start() {
    let mut c = conf::Conf::new();
    c.window_mode = c.window_mode.dimensions(1600 as f32, 900 as f32);
    c.window_setup = c.window_setup.vsync(false).samples(NumSamples::Four);

    let mut cb = ContextBuilder::new("Sandbox", "Uriopass").conf(c);

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }

    let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();

    let mut state = EngineState::new(ctx).unwrap();

    event::run(ctx, event_loop, &mut state).unwrap()
}
