#![deny(
    rustdoc::all,
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    unused_extern_crates,
    clippy::all,
    clippy::doc_markdown,
    clippy::wildcard_imports
)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix,
    clippy::blocks_in_if_conditions,
    clippy::upper_case_acronyms,
    clippy::must_use_candidate,
    missing_copy_implementations,
    missing_debug_implementations
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
