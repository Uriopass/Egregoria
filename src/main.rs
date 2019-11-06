use ggez::*;
use ggez::input::keyboard::{KeyCode, KeyMods};

struct State {
    dt: std::time::Duration,
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.dt = timer::average_delta(ctx);
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::from_rgb(0, 0, 0));
        graphics::set_window_title(ctx, format!("{} FPS", 1./self.dt.as_secs_f32()).as_str());


        timer::yield_now();
        graphics::present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _: KeyMods, _: bool) {
        if keycode == KeyCode::A {
            println!("A JUST PRESSED!!");
        }

    }
}

fn main() {
    let state = &mut State {
        dt: std::time::Duration::new(0, 0)
    };
    let c = conf::Conf::new();

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("hello_ggez", "Uriopass")
        .conf(c)
        .build()
        .unwrap();

    event::run(ctx, event_loop, state).unwrap();
}