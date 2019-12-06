use legion::prelude::*;

use engine::*;

use crate::engine::components::{Position, Velocity};
use crate::engine::resources::DeltaTime;

mod dijkstra;
mod engine;
mod humans;

fn main() {
    let universe = Universe::new();
    let mut world = universe.create_world();

    let speed_apply = SystemBuilder::new("update_pos")
        .with_query(<(Write<Position>, Read<Velocity>)>::query())
        .read_resource::<DeltaTime>()
        .build(|_, mut world, res, query| {
            let delta: f32 = (**res).0;
            for (mut pos, vel) in query.iter(&mut world) {
                pos.0 += vel.0 * delta;
            }
        });

    let schedule = Schedule::builder()
        .add_system(humans::setup(&mut world))
        .add_system(speed_apply)
        .build();
    engine::start(world, schedule);
}
