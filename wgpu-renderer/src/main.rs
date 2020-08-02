mod debug;
mod engine;
mod game_loop;
mod geometry;
mod rendering;

fn main() {
    env_logger::init();

    let mut ctx = engine::Context::new();

    let state = game_loop::State::new(&mut ctx);
    ctx.start(state);
}
