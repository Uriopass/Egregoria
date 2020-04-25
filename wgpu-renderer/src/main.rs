#![allow(dead_code)]

mod engine;
mod game_loop;
mod geometry;
mod rendering;

fn main() {
    let (mut ctx, el) = engine::Context::new();

    let state = game_loop::State::new(&mut ctx);

    ctx.start(state, el);
}
