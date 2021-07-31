#![allow(
    clippy::too_many_arguments,
    clippy::float_cmp,
    elided_lifetimes_in_paths,
    clippy::upper_case_acronyms,
    missing_copy_implementations,
    missing_debug_implementations
)]
#![deny(
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    unused_extern_crates
)]

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
