#![allow(clippy::too_many_arguments)]
#![allow(clippy::float_cmp)]
#![allow(elided_lifetimes_in_paths)]
#![allow(clippy::upper_case_acronyms)]

use crate::context::Context;
use common::logger::MyLog;

#[macro_use]
extern crate common;

#[macro_use]
extern crate egregoria;

#[macro_use]
mod uiworld;

mod audio;
mod context;
mod game_loop;
mod gui;
mod input;
mod network;
mod rendering;

fn main() {
    MyLog::init();

    let mut ctx = Context::new();

    let state = game_loop::State::new(&mut ctx);
    ctx.start(state);
}
