#![windows_subsystem = "windows"]

use crate::engine_interaction::{
    KeyboardInfo, MeshRenderEventReader, MouseButton, MouseInfo, RenderStats, TimeInfo,
};
use crate::geometry::gridstore::GridStore;
use crate::gui::Gui;
use crate::humans::HumanUpdate;
use crate::interaction::{
    FollowEntity, MovableSystem, MovedEvent, SelectableAuraSystem, SelectableSystem, SelectedEntity,
};
use crate::map_model::{LogicComponent, RoadGraphSynchronize, RoadGraphSynchronizeState};
use crate::physics::systems::KinematicsApply;
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Transform};
use crate::rendering::meshrender_component::MeshRender;
use crate::transportation::systems::TransportDecision;
use cgmath::InnerSpace;
use specs::{Dispatcher, DispatcherBuilder, Join, World, WorldExt};

#[macro_use]
pub mod gui;

pub mod engine_interaction;
pub mod geometry;
pub mod graphs;
pub mod humans;
pub mod interaction;
pub mod map_model;
pub mod physics;
pub mod rendering;
pub mod transportation;
pub use specs;
use specs::shrev::EventChannel;

pub fn dispatcher<'a>() -> Dispatcher<'a, 'a> {
    DispatcherBuilder::new()
        .with(HumanUpdate, "human update", &[])
        .with(TransportDecision, "car decision", &[])
        .with(SelectableSystem, "selectable", &[])
        .with(
            MovableSystem::default(),
            "movable",
            &["human update", "car decision", "selectable"],
        )
        .with(RoadGraphSynchronize, "rgs", &["movable"])
        .with(KinematicsApply, "speed apply", &["movable"])
        .with(
            SelectableAuraSystem::default(),
            "selectable aura",
            &["movable"],
        )
        .build()
}

pub fn ad_hoc_systems(world: &mut World) {
    let logic = world.read_component::<LogicComponent>();
    let trans = world.read_component::<Transform>();
    let mouse: &MouseInfo = &world.read_resource::<MouseInfo>();

    if mouse.just_pressed.contains(&MouseButton::Left) {
        for (t, l) in (&trans, &logic).join() {
            if (mouse.unprojected - t.position()).magnitude() < l.radius {
                let v = &l.on_click;
                v(world);
            }
        }
    }
}

pub fn setup(world: &mut World, dispatcher: &mut Dispatcher) {
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
    world.register::<LogicComponent>();

    // Event channels init
    let reader = MeshRenderEventReader(world.write_storage::<MeshRender>().register_reader());
    world.insert(reader);

    world.insert(EventChannel::<MovedEvent>::new());

    // Systems state init
    let s = RoadGraphSynchronizeState::new(world);
    world.insert(s);

    dispatcher.setup(world);
    map_model::setup(world);
    transportation::setup(world);
}
