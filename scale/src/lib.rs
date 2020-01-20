#![windows_subsystem = "windows"]

use ncollide2d::world::CollisionWorld;

use crate::cars::car_system::CarDecision;
use crate::cars::roads::RoadGraphSynchronize;
use crate::engine_interaction::{DeltaTime, KeyboardInfo};
use crate::gui::TestGui;
use crate::humans::HumanUpdate;
use crate::interaction::{MovableSystem, SelectableSystem, SelectedEntity};
use crate::physics::physics_components::Collider;
use crate::physics::physics_system::{KinematicsApply, PhysicsUpdate};
use crate::physics::PhysicsWorld;
use crate::rendering::meshrender_component::MeshRender;
use specs::{Dispatcher, DispatcherBuilder, World, WorldExt};

mod cars;
pub mod engine_interaction;
mod graphs;
pub mod gui;
mod humans;
mod interaction;
mod physics;
pub mod rendering;

pub fn dispatcher<'a>(world: &mut World) -> Dispatcher<'a, 'a> {
    DispatcherBuilder::new()
        .with(HumanUpdate, "human update", &[])
        .with(CarDecision, "car decision", &[])
        .with(
            MovableSystem::default(),
            "movable",
            &["human update", "car decision"],
        )
        .with(RoadGraphSynchronize::new(world), "rgs", &["movable"])
        .with(KinematicsApply, "speed apply", &["movable"])
        .with(PhysicsUpdate::default(), "physics", &["speed apply"])
        .with(SelectableSystem::default(), "selectable", &[])
        .build()
}

pub fn setup(world: &mut World, dispatcher: &mut Dispatcher) {
    let collision_world: PhysicsWorld = CollisionWorld::new(2.0);

    world.insert(DeltaTime(0.0));
    world.insert(collision_world);
    world.insert(KeyboardInfo::default());
    world.insert(TestGui);
    world.insert(SelectedEntity::default());

    world.register::<MeshRender>();
    world.register::<Collider>();

    dispatcher.setup(world);

    humans::setup(world);
    cars::setup(world);
}
