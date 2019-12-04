use std::env;
use std::path;

use crate::gsb::GSB;
use crate::shape_render::ShapeRenderer;
use crate::walking_simulation::HumanManager;
use ggez::graphics::*;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::nalgebra::Matrix4;

use ggez::conf::NumSamples;
use ggez::*;

mod camera;
mod dijkstra;
mod gsb;
mod shape_render;
mod walking_simulation;

#[allow(dead_code)]
fn draw_text<P>(ctx: &mut Context, text: &graphics::Text, pos: P) -> GameResult<()>
where
    P: Into<mint::Point2<f32>>,
{
    let mut new_pos = pos.into();
    new_pos.y += text.height(ctx) as f32;
    let trans = graphics::DrawParam::new().dest(new_pos).scale([1., -1.]);
    graphics::draw(ctx, text, trans)
}

fn draw_image<P>(ctx: &mut Context, image: &graphics::Image, pos: P) -> GameResult<()>
where
    P: Into<mint::Point2<f32>>,
{
    let mut new_pos = pos.into();
    new_pos.y += image.height() as f32;
    let trans = graphics::DrawParam::new().dest(new_pos).scale([1., -1.]);
    graphics::draw(ctx, image, trans)
}

struct State {
    gsb: GSB,
    time: f32,
    hm: walking_simulation::HumanManager,
}

#[derive(PartialEq)]
pub enum EVACOLOR {
    WHITE,
    RED,
    NONE,
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        println!("{}", filesystem::resources_dir(ctx).display());

        //let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        //let text = graphics::Text::new(("Hello world!", font, 48.0));
        //let test: Image = graphics::Image::new(ctx, "/test.png")?;

        graphics::set_resizable(ctx, true)?;
        Ok(State {
            gsb: gsb::GSB::new(),
            time: 0.,
            hm: HumanManager::new(100),
        })
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        self.time += delta;
        for _ in 0..2 {
            self.hm.update(ctx, &self.gsb, delta);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.gsb.clear(ctx);
        self.gsb.easy_camera_movement(ctx);
        self.gsb.update(ctx);

        let mut sr = ShapeRenderer::begin(self.gsb.get_screen_box());
        self.hm.draw(&mut sr);
        sr.end(ctx)?;

        graphics::pop_transform(ctx);
        graphics::apply_transformations(ctx)?;
        graphics::present(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        self.gsb.easy_camera_movement_keys(ctx, keycode);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.gsb.resize(ctx, width, height);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        if y > 0. {
            self.gsb.easy_camera_movement_keys(ctx, KeyCode::Add);
        }
        if y < 0. {
            self.gsb.easy_camera_movement_keys(ctx, KeyCode::Subtract);
        }
    }
}

fn main() {
    let mut c = conf::Conf::new();
    c.window_mode = c.window_mode.dimensions(1600 as f32, 900 as f32);
    c.window_setup = c.window_setup.vsync(false).samples(NumSamples::Four);

    let mut cb = ContextBuilder::new("hello_ggez", "Uriopass").conf(c);

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }

    let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();

    let mut state = State::new(ctx).unwrap();

    event::run(ctx, event_loop, &mut state).unwrap()
}
