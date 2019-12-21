#![windows_subsystem = "windows"]

use engine::ncollide2d::world::CollisionWorld;
use engine::specs::{DispatcherBuilder, World, WorldExt};

use crate::cars::car_system::CarDecision;
use crate::cars::RoadNodeComponent;
use crate::humans::HumanUpdate;
use engine::components::{Collider, MeshRenderComponent};
use engine::resources::DeltaTime;
use engine::systems::{KinematicsApply, MovableSystem, PhysicsUpdate};
use engine::PhysicsWorld;

mod cars;
mod graphs;
mod humans;

trait Id {
    fn id() -> &'static str;
}
impl<T: ?Sized> Id for T {
    fn id() -> &'static str {
        std::any::type_name::<T>()
    }
}

fn main() {
    let collision_world: PhysicsWorld = CollisionWorld::new(2.0);

    let mut world = World::new();

    world.insert(DeltaTime(0.0));
    world.insert(collision_world);

    world.register::<MeshRenderComponent>();
    world.register::<Collider>();
    world.register::<RoadNodeComponent>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, HumanUpdate::id(), &[])
        .with(CarDecision, CarDecision::id(), &[])
        .with(
            MovableSystem::default(),
            MovableSystem::id(),
            &[HumanUpdate::id(), CarDecision::id()],
        )
        .with(
            KinematicsApply,
            KinematicsApply::id(),
            &[MovableSystem::id()],
        )
        .with(PhysicsUpdate, PhysicsUpdate::id(), &[KinematicsApply::id()])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world);
    cars::setup(&mut world);

    engine::start(world, dispatcher);
}
