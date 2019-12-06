use engine::*;

use crate::engine::components::{CircleRender, Position, Velocity};
use crate::engine::resources::DeltaTime;
use crate::humans::HumanUpdate;
use specs::prelude::*;

mod dijkstra;
mod engine;
mod humans;

struct SpeedApply;

impl<'a> System<'a> for SpeedApply {
    type SystemData = (
        Read<'a, DeltaTime>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Velocity>,
    );

    fn run(&mut self, (delta, mut pos, vel): Self::SystemData) {
        let delta = delta.0;

        for (vel, pos) in (&vel, &mut pos).join() {
            pos.0 += vel.0 * delta;
        }
    }
}

fn main() {
    let mut world = World::new();

    world.insert(DeltaTime(0.));
    world.register::<CircleRender>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human_update", &[])
        .with(SpeedApply, "hello_world", &["human_update"])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world);

    engine::start(world, dispatcher);
}
