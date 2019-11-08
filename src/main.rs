use std::env;
use std::path;

use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::*;

mod camera;

struct State {
    dt: std::time::Duration,
    text: graphics::Text,
    camera: camera::Camera,
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        // The ttf file will be in your resources directory. Later, we
        // will mount that directory so we can omit it in the path here.

        println!("{}", filesystem::resources_dir(ctx).display());

        let font = graphics::Font::new(ctx, "/bmonofont-i18n.ttf")?;
        let text = graphics::Text::new(("Hello world!", font, 48.0));

        graphics::set_resizable(ctx, true)?;
        let c = camera::Camera::new(400., 300.0);

        Ok(State {
            text,
            dt: std::time::Duration::new(0, 0),
            camera: c,
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
        graphics::set_projection(ctx, self.camera.projection);
        graphics::clear(ctx, graphics::Color::from_rgb(0, 0, 0));
        graphics::set_window_title(ctx, format!("{} FPS", 1. / self.dt.as_secs_f32()).as_str());

        let a = timer::ticks(ctx) / 10;

        let x = a as f32;
        let y = a as f32;

        draw_text(ctx, &self.text, [x, y])?;
        graphics::present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        if keycode == KeyCode::A {
            println!("A JUST PRESSED!!");
        }
    }

    fn resize_event(&mut self, _ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.camera.update();
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
