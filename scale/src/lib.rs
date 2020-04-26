#![windows_subsystem = "windows"]
#![allow(clippy::unreadable_literal)]

use crate::engine_interaction::{KeyboardInfo, RenderStats, TimeInfo};
use crate::geometry::gridstore::GridStore;
use crate::gui::Gui;
use crate::interaction::{
    FollowEntity, MovableSystem, MovedEvent, SelectableAuraSystem, SelectableSystem, SelectedEntity,
};
use crate::map_model::{MapUIState, MapUISystem};
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

pub fn setup<'a>(world: &mut World) -> Dispatcher<'a, 'a> {
    let mut dispatch = DispatcherBuilder::new()
        .with(VehicleDecision, "car decision", &[])
        .with(PedestrianDecision, "pedestrian decision", &[])
        .with(SelectableSystem, "selectable", &[])
        .with(
            MovableSystem::default(),
            "movable",
            &["car decision", "pedestrian decision", "selectable"],
        )
        .with(MapUISystem, "rgs", &["movable"])
        .with(KinematicsApply, "speed apply", &["movable"])
        .with(
            SelectableAuraSystem::default(),
            "selectable aura",
            &["movable"],
        )
        .build();

    let collision_world: CollisionWorld = GridStore::new(50);

    // Resources init
    world.insert(TimeInfo::default());
    world.insert(collision_world);
    world.insert(KeyboardInfo::default());
    world.insert(Gui::default());
    world.insert(SelectedEntity::default());
    world.insert(FollowEntity::default());
    world.insert(RenderStats::default());

    world.register::<Collider>();
    world.register::<MeshRender>();
    world.register::<AssetRender>();

    // Event channels init
    world.insert(EventChannel::<MovedEvent>::new());

    // Systems state init
    let s = MapUIState::new(world);
    world.insert(s);

    dispatch.setup(world);

    map_model::setup(world);
    vehicles::setup(world);
    pedestrians::setup(world);

    dispatch
}
