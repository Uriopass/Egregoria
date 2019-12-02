use std::env;
use std::path;

use crate::gsb::GSB;
use crate::shape_render::ShapeRenderer;
use ggez::graphics::*;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::*;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

mod camera;
mod car;
mod dijkstra;
mod gsb;
mod shape_render;
mod walking_simulation;

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
    text: graphics::Text,
    gsb: GSB,
    time: f32,
    cars: Vec<car::Car>,
    test: graphics::Image,
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        println!("{}", filesystem::resources_dir(ctx).display());

        let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        let text = graphics::Text::new(("Hello world!", font, 48.0));
        let test: Image = graphics::Image::new(ctx, "/test.png")?;

        graphics::set_resizable(ctx, true)?;
        Ok(State {
            text,
            gsb: gsb::GSB::new(),
            time: 0.,
            cars: (0..100000)
                .into_iter()
                .map(|_| car::Car {
                    position: [0., 0.].into(),
                })
                .collect(),
            test,
        })
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let delta = timer::delta(ctx).as_secs_f32();
        self.time += delta;

        let mut rng = SmallRng::from_entropy();

        for c in self.cars.iter_mut() {
            c.position.x = rng.gen::<f32>() * 1000.;
            c.position.y = rng.gen::<f32>() * 1000.;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.gsb.clear(ctx);
        self.gsb.easy_camera_movement(ctx);
        self.gsb.update(ctx);

        let _lol = self.gsb.unproject_mouse_click(ctx);

        let x = 50.0 * (self.time as f32).cos();
        let y = 50.0 * (1.5 * self.time as f32 + 0.5).sin();

        let mut sr = ShapeRenderer::begin();
        sr.color = graphics::WHITE;
        //sr.mode = DrawMode::stroke(1.0);
        for c in self.cars.iter() {
            sr.draw_rect(c.position, 1., 1.);
        }
        sr.end(ctx)?;
        draw_text(ctx, &self.text, [0., 0.])?;
        draw_image(ctx, &self.test, [0.0, 0.0])?;
        graphics::present(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        self.gsb.easy_camera_movement_keys(ctx, keycode);
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.gsb.resize(ctx, width, height);
    }
}

fn main() {
    let mut c = conf::Conf::new();
    c.window_setup = c.window_setup.vsync(false);

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
