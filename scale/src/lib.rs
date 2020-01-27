#![windows_subsystem = "windows"]

use crate::cars::car_system::CarDecision;
use crate::cars::map::RoadGraphSynchronize;
use crate::cars::CarMarker;
use crate::engine_interaction::{KeyboardInfo, MeshRenderEventReader, TimeInfo};
use crate::gui::Gui;
use crate::humans::HumanUpdate;
use crate::interaction::{
    FollowEntity, MovableSystem, SelectableAuraSystem, SelectableSystem, SelectedEntity,
};
use crate::physics::physics_components::Collider;
use crate::physics::physics_system::{KinematicsApply, PhysicsUpdate};
use crate::physics::PhysicsWorld;
use crate::rendering::meshrender_component::MeshRender;
use ncollide2d::world::CollisionWorld;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use specs::{Dispatcher, DispatcherBuilder, World, WorldExt};

pub mod cars;
pub mod engine_interaction;
pub mod graphs;
pub mod gui;
pub mod humans;
pub mod interaction;
pub mod physics;
pub mod rendering;

pub fn dispatcher<'a>(world: &mut World) -> Dispatcher<'a, 'a> {
    world.register::<MeshRender>();
    let reader = MeshRenderEventReader(world.write_storage::<MeshRender>().register_reader());
    world.insert(reader);

    DispatcherBuilder::new()
        .with(HumanUpdate, "human update", &[])
        .with(CarDecision, "car decision", &[])
        .with(SelectableSystem, "selectable", &[])
        .with(
            MovableSystem::default(),
            "movable",
            &["human update", "car decision", "selectable"],
        )
        .with(RoadGraphSynchronize::new(world), "rgs", &["movable"])
        .with(KinematicsApply, "speed apply", &["movable"])
        .with(PhysicsUpdate::default(), "physics", &["speed apply"])
        .with(
            SelectableAuraSystem::default(),
            "selectable aura",
            &["movable"],
        )
        .build()
}

pub fn setup(world: &mut World, dispatcher: &mut Dispatcher) {
    let collision_world: PhysicsWorld = CollisionWorld::new(2.0);

    world.insert(TimeInfo::default());
    world.insert(collision_world);
    world.insert(KeyboardInfo::default());
    world.insert(Gui::default());
    world.insert(SelectedEntity::default());
    world.insert(FollowEntity::default());

    world.register::<Collider>();
    world.register::<SimpleMarker<CarMarker>>();

    world.insert(SimpleMarkerAllocator::<CarMarker>::default());

    dispatcher.setup(world);
    humans::setup(world);
    cars::setup(world);
}
