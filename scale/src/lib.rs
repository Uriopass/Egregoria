#![allow(clippy::unreadable_literal)]
#![allow(clippy::block_in_if_condition_stmt)]
#![allow(clippy::too_many_arguments)]

use crate::engine_interaction::{KeyboardInfo, RenderStats, TimeInfo};
use crate::gui::Gui;
use crate::interaction::{
    BulldozerResource, BulldozerSystem, DeletedEvent, FollowEntity, InspectedAuraSystem,
    InspectedEntity, MovableSystem, MovedEvent, RoadEditorResource, RoadEditorSystem,
    SelectableSystem,
};
use crate::interaction::{IntersectionComponent, RoadBuildResource, RoadBuildSystem};
use crate::map_interaction::{ItinerarySystem, ParkingManagement};
use crate::pedestrians::PedestrianDecision;
use crate::physics::systems::KinematicsApply;
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Transform};
use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use crate::vehicles::systems::VehicleDecision;
use specs::shrev::EventChannel;
use specs::world::EntitiesRes;
use specs::{Dispatcher, DispatcherBuilder, LazyUpdate, World, WorldExt};

#[macro_use]
pub mod utils;

#[macro_use]
pub mod log;

#[macro_use]
pub mod gui;

pub mod engine_interaction;
pub mod interaction;
pub mod map_interaction;
pub mod pedestrians;
pub mod physics;
pub mod rand_provider;
pub mod rendering;
pub mod vehicles;

pub use imgui;
pub use rand_provider::RandProvider;
pub use specs;

pub struct ScaleState<'a> {
    pub world: World,
    dispatcher: Dispatcher<'a, 'a>,
}

const RNG_SEED: u64 = 123;

impl<'a> ScaleState<'a> {
    pub fn run(&mut self) {
        crate::log::clear();
        let t = std::time::Instant::now();
        self.dispatcher.dispatch_seq(&self.world);
        self.dispatcher.dispatch_thread_local(&self.world);
        self.world.maintain();
        self.world.write_resource::<RenderStats>().update_time = t.elapsed().as_secs_f32();
    }

    pub fn setup() -> ScaleState<'a> {
        let mut world = World::empty();

        println!("Seed is {}", RNG_SEED);

        // Basic resources init
        world.insert(EntitiesRes::default());
        world.insert(TimeInfo::default());
        world.insert(CollisionWorld::new(50));
        world.insert(KeyboardInfo::default());
        world.insert(Gui::default());
        world.insert(InspectedEntity::default());
        world.insert(FollowEntity::default());
        world.insert(RenderStats::default());
        world.insert(RandProvider::new(RNG_SEED));
        world.insert(LazyUpdate::default());
        world.insert(ParkingManagement::default());

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

        world.insert(map_model::load());
        vehicles::setup(&mut world);
        pedestrians::setup(&mut world);

        Self { world, dispatcher }
    }
}
