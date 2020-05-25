#![allow(clippy::unreadable_literal)]
#![allow(clippy::block_in_if_condition_stmt)]

use crate::engine_interaction::{KeyboardInfo, RenderStats, TimeInfo};
use crate::gui::Gui;
use crate::interaction::{
    DeletedEvent, FollowEntity, InspectedAuraSystem, InspectedEntity, MovableSystem, MovedEvent,
    SelectableSystem,
};
use crate::interaction::{RoadBuildState, RoadBuildSystem};
use crate::pedestrians::PedestrianDecision;
use crate::physics::systems::KinematicsApply;
use crate::physics::Collider;
use crate::physics::CollisionWorld;
use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::systems::VehicleDecision;
use specs::{Dispatcher, DispatcherBuilder, World, WorldExt};

#[macro_use]
pub mod utils;

#[macro_use]
pub mod geometry;

#[macro_use]
pub mod gui;

pub mod engine_interaction;
pub mod graphs;
pub mod interaction;
pub mod map_model;
pub mod pedestrians;
pub mod physics;
pub mod rendering;
pub mod vehicles;

pub use imgui;
pub use specs;
use specs::shrev::EventChannel;
use specs::world::EntitiesRes;

pub fn setup<'a>(world: &mut World) -> Dispatcher<'a, 'a> {
    let collision_world: CollisionWorld = CollisionWorld::new(50);

    // Resources init
    world.insert(EntitiesRes::default());
    world.insert(TimeInfo::default());
    world.insert(collision_world);
    world.insert(KeyboardInfo::default());
    world.insert(Gui::default());
    world.insert(InspectedEntity::default());
    world.insert(FollowEntity::default());
    world.insert(RenderStats::default());

    world.register::<Collider>();
    world.register::<MeshRender>();
    world.register::<AssetRender>();

    // Event channels init
    world.insert(EventChannel::<MovedEvent>::new());
    world.insert(EventChannel::<DeletedEvent>::new());

    // Systems state init
    let s = RoadBuildState::new(world);
    world.insert(s);

    // Dispatcher init
    let mut dispatch = DispatcherBuilder::new()
        .with(VehicleDecision, "car decision", &[])
        .with(PedestrianDecision, "pedestrian decision", &[])
        .with(SelectableSystem, "selectable", &[])
        .with(
            MovableSystem::default(),
            "movable",
            &["car decision", "pedestrian decision", "selectable"],
        )
        .with(RoadBuildSystem, "rgs", &["movable"])
        .with(KinematicsApply::new(world), "speed apply", &["movable"])
        .with(
            InspectedAuraSystem::default(),
            "selectable aura",
            &["movable"],
        )
        .build();

    dispatch.setup(world);

    map_model::setup(world);
    vehicles::setup(world);
    pedestrians::setup(world);

    dispatch
}
