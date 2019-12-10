use engine::*;

use crate::engine::components::{CircleRender, Collider, LineRender};
use crate::engine::resources::DeltaTime;
use crate::engine::systems::{MovableSystem, SpeedApply};
use crate::humans::HumanUpdate;
use ncollide2d::world::CollisionWorld;
use specs::prelude::*;

mod dijkstra;
mod engine;
mod geometry;
mod humans;

type PhysicsWorld = CollisionWorld<f32, Entity>;

fn main() {
    let mut collision_world: PhysicsWorld = CollisionWorld::new(0.02);

    let mut world = World::new();

    world.insert(DeltaTime(0.));

    world.register::<CircleRender>();
    world.register::<LineRender>();
    world.register::<Collider>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human_update", &[])
        .with(SpeedApply, "speed_apply", &["human_update"])
        .with(MovableSystem::default(), "movable", &[])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world, &mut collision_world);

    world.insert::<PhysicsWorld>(collision_world);

    engine::start(world, dispatcher);
}
