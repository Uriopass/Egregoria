use std::env;
use std::path;

use ggez::conf::NumSamples;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::*;

use crate::engine::camera_handler::CameraHandler;
use crate::engine::components::{CircleRender, Position};
use crate::engine::render_context::RenderContext;
use crate::engine::resources::{DeltaTime, MouseInfo};
use ggez::graphics::{DrawMode, DrawParam, MeshBuilder};
use ggez::input::mouse::MouseButton;
use specs::prelude::*;
use std::collections::HashSet;
use std::iter::FromIterator;

pub mod camera;
pub mod camera_handler;
pub mod components;
pub mod render_context;
pub mod resources;
pub mod shape_render;
pub mod systems;

pub struct EngineState<'a> {
    pub world: World,
    pub dispatch: Dispatcher<'a, 'a>,
    pub time: f32,
    pub cam: CameraHandler,
}

impl<'a> EngineState<'a> {
    fn new(
        world: World,
        dispatch: Dispatcher<'a, 'a>,
        ctx: &mut Context,
    ) -> GameResult<EngineState<'a>> {
        println!("{}", filesystem::resources_dir(ctx).display());

        //let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        //let text = graphics::Text::new(("Hello world!", font, 48.0));
        //let test: Image = graphics::Image::new(ctx, "/test.png")?;

        graphics::set_resizable(ctx, true)?;
        Ok(EngineState {
            world,
            dispatch,
            time: 0.,
            cam: CameraHandler::new(),
        })
    }
}

impl<'a> ggez::event::EventHandler for EngineState<'a> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        self.time += delta;
        *self.world.write_resource() = DeltaTime(delta * 5.);
        *self.world.write_resource() = MouseInfo {
            unprojected: self.cam.unproject_mouse_click(ctx),
            buttons: HashSet::from_iter(
                vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle]
                    .into_iter()
                    .filter(|x| ggez::input::mouse::button_pressed(ctx, *x)),
            ),
        };

        self.dispatch.run_now(&mut self.world);
        self.world.maintain();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.cam.easy_camera_movement(ctx);
        self.cam.update(ctx);

        let mut rc = RenderContext::new(&mut self.cam, ctx);
        rc.clear();

        let pos = self.world.read_component::<Position>();
        let circle_render = self.world.read_component::<CircleRender>();

        for (pos, size) in (&pos, &circle_render).join() {
            let pos = pos.0;
            rc.sr.draw_circle([pos.x, pos.y], size.radius);
        }

        rc.finish()?;
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

pub fn start<'a>(world: World, schedule: Dispatcher<'a, 'a>) {
    let mut c = conf::Conf::new();
    c.window_mode = c.window_mode.dimensions(800 as f32, 600 as f32);
    c.window_setup = c.window_setup.vsync(false).samples(NumSamples::Four);

    let mut cb = ContextBuilder::new("Sandbox", "Uriopass").conf(c);

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }

    let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();

    let mut state = EngineState::new(world, schedule, ctx).unwrap();

    event::run(ctx, event_loop, &mut state).unwrap()
}
