use crate::context::Context;
use common::logger::MyLog;

#[macro_use]
extern crate common;

#[macro_use]
extern crate egregoria;
extern crate core;

#[macro_use]
extern crate inline_tweak;

#[macro_use]
mod uiworld;

mod audio;
mod context;
mod game_loop;
mod gui;
mod init;
mod input;
mod inputmap;
mod network;
mod rendering;

fn main() {
    profiling::register_thread!("Main Thread");

    MyLog::init();
    init::init();

    let mut ctx = Context::new();

    let state = game_loop::State::new(&mut ctx);
    ctx.start(state);
}
