#![windows_subsystem = "windows"]

use engine::ncollide2d::world::CollisionWorld;
use engine::specs::{DispatcherBuilder, World, WorldExt};

use crate::cars::car_graph::RoadGraphSynchronize;
use crate::cars::car_system::CarDecision;
use crate::humans::HumanUpdate;
use engine::components::{Collider, MeshRenderComponent};
use engine::resources::{DeltaTime, KeyboardInfo};
use engine::systems::{KinematicsApply, MovableSystem, PhysicsUpdate};
use engine::PhysicsWorld;

mod cars;
mod graphs;
mod humans;

fn main() {
    let collision_world: PhysicsWorld = CollisionWorld::new(2.0);

    let mut world = World::new();

    world.insert(DeltaTime(0.0));
    world.insert(collision_world);
    world.insert(KeyboardInfo::default());

    world.register::<MeshRenderComponent>();
    world.register::<Collider>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human update", &[])
        .with(CarDecision, "car decision", &[])
        .with(
            MovableSystem::new(),
            "movable",
            &["human update", "car decision"],
        )
        .with(RoadGraphSynchronize, "rgs", &["movable"])
        .with(KinematicsApply, "speed apply", &["movable"])
        .with(PhysicsUpdate::default(), "physics", &["speed apply"])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world);
    cars::setup(&mut world);

    engine::start(world, dispatcher);
}
