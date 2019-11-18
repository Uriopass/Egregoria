use std::env;
use std::path;

use crate::gsb::GSB;
use crate::shape_render::ShapeRenderer;
use ggez::graphics::*;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::*;

mod camera;
mod gsb;
mod shape_render;

struct State {
    text: graphics::Text,
    gsb: GSB,
    time: f32,
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.

        println!("{}", filesystem::resources_dir(ctx).display());

        let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        let text = graphics::Text::new(("Hello world!", font, 48.0));

        graphics::set_resizable(ctx, true)?;
        Ok(State {
            text,
            gsb: gsb::GSB::new(),
            time: 0.,
        })
    }
}

fn draw_text<P>(ctx: &mut Context, text: &graphics::Text, pos: P) -> GameResult<()>
where
    P: Into<mint::Point2<f32>>,
{
    let mut new_pos = pos.into();
    new_pos.y += text.height(ctx) as f32;
    let trans = graphics::DrawParam::new().dest(new_pos).scale([1., -1.]);
    graphics::draw(ctx, text, trans)
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.time += timer::delta(ctx).as_secs_f32();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.gsb.clear(ctx);
        self.gsb.easy_camera_movement(ctx);
        self.gsb.update(ctx);

        let lol = self.gsb.unproject_mouse_click(ctx);

        let x = 50.0 * (self.time as f32).cos();
        let y = 50.0 * (1.5 * self.time as f32 + 0.5).sin();

        let mut sr = ShapeRenderer::begin();
        //sr.mode = DrawMode::fill();

        for _i in 0..10000 {
            sr.draw_rect(lol, 10., 10.);
            sr.draw_rect([x, y], 10., 10.);
        }
        sr.end(ctx)?;

        draw_text(ctx, &self.text, [0., 0.])?;
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
