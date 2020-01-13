use std::env;
use std::path;

use ggez::conf::NumSamples;
use ggez::{conf, event, ContextBuilder};
use specs::{World, WorldExt};

use crate::game_loop::EngineState;
mod game_loop;
mod geometry;
mod gui;
mod rendering;

fn main() {
    let mut world = World::new();
    let mut schedule = scale::dispatcher(&mut world);
    scale::setup(&mut world, &mut schedule);

    let mut c = conf::Conf::new();
    if cfg!(target_os = "windows") {
        c.window_mode = c.window_mode.dimensions(1600.0, 900.0);
    } else {
        c.window_mode = c.window_mode.dimensions(800.0, 600.0);
    }
    c.window_setup = c.window_setup.vsync(false).samples(NumSamples::Four);

    let mut cb = ContextBuilder::new("Sandbox", "Uriopass").conf(c);

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }

    let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();

    let mut state: EngineState = game_loop::EngineState::new(world, schedule, ctx).unwrap();

    state.cam.camera.zoom = 10.0;
    state.cam.camera.position.x = 50.0;
    state.cam.camera.position.y = 50.0;

    event::run(ctx, event_loop, &mut state).unwrap()
}
