#![allow(clippy::type_complexity)]

#[macro_use]
extern crate common;

#[macro_use]
extern crate simulation;

#[allow(unused_imports)]
#[macro_use]
extern crate inline_tweak;

#[macro_use]
mod uiworld;

mod audio;
mod game_loop;
mod gui;
mod init;
mod inputmap;
mod network;
mod rendering;

fn main() {
    #[cfg(feature = "profile")]
    profiling::tracy_client::Client::start();
    profiling::register_thread!("Main Thread");

    init::init();
    engine::framework::start::<game_loop::State>();
}
