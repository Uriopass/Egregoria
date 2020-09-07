#![allow(clippy::unreadable_literal)]
#![allow(clippy::block_in_if_condition_stmt)]
#![allow(clippy::too_many_arguments)]

use crate::engine_interaction::{KeyboardInfo, RenderStats, TimeInfo};
use crate::frame_log::FrameLog;
use crate::gui::Gui;
use crate::interaction::{
    BulldozerResource, BulldozerSystem, DeletedEvent, FollowEntity, InspectedAuraSystem,
    InspectedEntity, MovableSystem, MovedEvent, RoadEditorResource, RoadEditorSystem,
    SelectableSystem,
};
use crate::interaction::{IntersectionComponent, RoadBuildResource, RoadBuildSystem};
use crate::lua::scenario_runner::{RunningScenario, RunningScenarioSystem};
use crate::map_dynamic::{ItinerarySystem, ParkingManagement};
use crate::pedestrians::PedestrianDecision;
use crate::physics::systems::KinematicsApply;
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Transform};
use crate::rendering::assets::AssetRender;
use crate::rendering::immediate::ImmediateDraw;
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::systems::VehicleDecision;
use map_model::{Map, SerializedMap};
use rand_provider::RandProvider;
use specs::shrev::EventChannel;
use specs::world::EntitiesRes;
use specs::{Dispatcher, DispatcherBuilder, LazyUpdate, World, WorldExt};
use std::io::Write;

#[macro_use]
extern crate log as extern_log;

#[macro_use]
pub mod utils;

#[macro_use]
mod frame_log;

#[macro_use]
pub mod gui;

mod companies;
pub mod engine_interaction;
pub mod interaction;
pub mod lua;
pub mod map_dynamic;
mod pedestrians;
pub mod physics;
pub mod rand_provider;
pub mod rendering;
mod saveload;
mod souls;
mod vehicles;

use crate::souls::Souls;
pub use imgui;
pub use specs;

pub struct EgregoriaState {
    pub world: World,
    dispatcher: Dispatcher<'static, 'static>,
}

const RNG_SEED: u64 = 123;

impl EgregoriaState {
    pub fn run(&mut self) {
        self.world.read_resource::<FrameLog>().clear();
        let t = std::time::Instant::now();
        self.dispatcher.dispatch_seq(&self.world);
        self.dispatcher.dispatch_thread_local(&self.world);
        self.world.maintain();
        self.world
            .write_resource::<RenderStats>()
            .add_update_time(t.elapsed().as_secs_f32());
    }

    pub fn init() -> EgregoriaState {
        let mut world = World::empty();

        info!("Seed is {}", RNG_SEED);

        // Basic resources init
        world.insert(EntitiesRes::default());
        world.insert(TimeInfo::default());
        world.insert(CollisionWorld::new(50));
        world.insert(KeyboardInfo::default());
        world.insert(InspectedEntity::default());
        world.insert(FollowEntity::default());
        world.insert(RenderStats::default());
        world.insert(RandProvider::new(RNG_SEED));
        world.insert(LazyUpdate::default());
        world.insert(ParkingManagement::default());
        world.insert(FrameLog::default());
        world.insert(RunningScenario::default());
        world.insert(ImmediateDraw::default());
        world.insert(Souls::default());

        world.register::<Transform>();
        world.register::<Collider>();
        world.register::<MeshRender>();
        world.register::<AssetRender>();
        world.register::<IntersectionComponent>();

        // Event channels init
        world.insert(EventChannel::<MovedEvent>::new());
        world.insert(EventChannel::<DeletedEvent>::new());

        // Systems state init
        let s = RoadBuildResource::new(&mut world);
        world.insert(s);

        let s = RoadEditorResource::new(&mut world);
        world.insert(s);

        let s = BulldozerResource::new(&mut world);
        world.insert(s);

        // Dispatcher init
        let mut dispatcher = DispatcherBuilder::new()
            .with(SelectableSystem, "selectable", &[])
            .with(RoadBuildSystem, "rgs", &[])
            .with(RoadEditorSystem, "res", &[])
            .with(BulldozerSystem, "bull", &[])
            .with(ItinerarySystem, "itinerary", &["rgs", "res", "bull"])
            .with(VehicleDecision, "car", &["itinerary"])
            .with(PedestrianDecision, "pedestrian", &["itinerary"])
            .with(RunningScenarioSystem, "scenario", &[])
            .with(
                MovableSystem::default(),
                "movable",
                &["car", "pedestrian", "selectable"],
            )
            .with(
                KinematicsApply::new(&mut world),
                "speed apply",
                &["movable"],
            )
            .with(
                InspectedAuraSystem::default(),
                "selectable aura",
                &["movable"],
            )
            .build();

        dispatcher.setup(&mut world);

        Self { world, dispatcher }
    }
}

pub fn load_from_disk(world: &mut World) {
    let map: Map = saveload::load_or_default::<map_model::SerializedMap>("map").into();
    world.insert(map);
    vehicles::setup(world);
    pedestrians::setup(world);

    world.insert(crate::saveload::load_or_default::<Gui>("gui"));
}

pub fn save_to_disk(world: &mut World) {
    let _ = std::io::stdout().flush();
    crate::saveload::save(&*world.read_resource::<Gui>(), "gui");
    crate::vehicles::save(world);
    crate::saveload::save(&SerializedMap::from(&*world.read_resource::<Map>()), "map");
}
