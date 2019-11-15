use std::env;
use std::path;

use ggez::graphics::*;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::*;
use crate::gsb::GSB;

mod camera;
mod gsb;
mod shape_render;

struct State {
    dt: std::time::Duration,
    text: graphics::Text,
    gsb: GSB,
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
            dt: std::time::Duration::new(0, 0),
            gsb: gsb::GSB::new(),
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
        self.dt = timer::average_delta(ctx);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.gsb.clear(ctx);
        self.gsb.easy_camera_movement(ctx);

        let lol = self.gsb.unproject_mouse_click(ctx);
        let circle = ggez::graphics::MeshBuilder::new()
            .circle(
                DrawMode::fill(),
                [0., 0.],
                20.,
                0.1,
                Color::new(1., 1., 1., 1.),
            )
            .build(ctx)?;

        let a = timer::ticks(ctx);

        let x = 50.0*((a as f32)/10.).cos();
        let y = 50.0*((1.5*a as f32 + 0.5)/7.).sin();

        graphics::draw(
            ctx,
            &circle,
            graphics::DrawParam::new().dest([lol.x, lol.y]),
        )?;
        graphics::draw(ctx, &circle, graphics::DrawParam::new().dest([x, y]))?;

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
    let c = conf::Conf::new();

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
