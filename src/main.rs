use engine::*;

use crate::engine::components::{CircleRender, LineRender};
use crate::engine::resources::DeltaTime;
use crate::engine::systems::{CollisionSystem, MovableSystem, SpeedApply};
use crate::humans::HumanUpdate;

use specs::prelude::*;

mod dijkstra;
mod engine;
mod geometry;
mod humans;

fn main() {
    let mut world = World::new();

    world.insert(DeltaTime(0.));

    world.register::<CircleRender>();
    world.register::<LineRender>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human_update", &[])
        .with(SpeedApply, "speed_apply", &["human_update"])
        .with(CollisionSystem, "collision_system", &["speed_apply"])
        .with(MovableSystem::default(), "movable", &[])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world);

    engine::start(world, dispatcher);
}
