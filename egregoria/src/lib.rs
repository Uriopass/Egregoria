#![allow(clippy::unreadable_literal)]
#![allow(clippy::block_in_if_condition_stmt)]
#![allow(clippy::too_many_arguments)]

use crate::engine_interaction::{KeyboardInfo, MouseInfo, RenderStats, TimeInfo};
use crate::frame_log::FrameLog;
use crate::gui::Gui;
use crate::interaction::{
    bulldozer_system, movable_system, roadbuild_system, roadeditor_system,
    selectable_cleanup_system, selectable_select_system, BulldozerResource, FollowEntity,
    InspectedAura, InspectedEntity, RoadEditorResource, Tool,
};
use crate::interaction::{inspected_aura_system, MovableSystem, RoadBuildResource};
use crate::lua::scenario_runner::{run_scenario_system, RunningScenario};
use crate::map_dynamic::{itinerary_update_system, ParkingManagement};
use crate::pedestrians::pedestrian_decision_system;
use crate::physics::systems::{
    coworld_maintain_system, coworld_synchronize_system, kinematics_apply_system,
};
use crate::physics::Collider;
use crate::physics::CollisionWorld;
use crate::rendering::immediate::ImmediateDraw;
use crate::souls::Souls;
use crate::vehicles::systems::{vehicle_cleanup_system, vehicle_decision_system};
use crate::vehicles::VehicleComponent;
pub use imgui;
use legion::storage::Component;
use legion::systems::Resource;
use legion::{Entity, IntoQuery, Resources, Schedule, World};
use map_model::{Map, SerializedMap};
pub(crate) use par_command_buffer::ParCommandBuffer;
use rand_provider::RandProvider;
use std::io::Write;
use std::ops::{Deref, DerefMut};

#[macro_use]
extern crate log as extern_log;

#[macro_use]
pub mod utils;

#[macro_use]
mod frame_log;

#[macro_use]
pub mod gui;

pub mod engine_interaction;
pub mod interaction;
pub mod lua;
pub mod map_dynamic;
mod par_command_buffer;
mod pedestrians;
pub mod physics;
pub mod rand_provider;
pub mod rendering;
mod saveload;
mod souls;
mod vehicles;

pub use legion;

pub struct Egregoria {
    pub world: World,
    resources: Resources,
    schedule: Schedule,
}

pub struct Deleted<T>(Vec<T>);
impl<T> Default for Deleted<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

const RNG_SEED: u64 = 123;

impl Egregoria {
    pub fn run(&mut self) {
        self.read::<FrameLog>().clear();
        let t = std::time::Instant::now();
        self.schedule.execute(&mut self.world, &mut self.resources);
        ParCommandBuffer::apply(self);
        self.write::<RenderStats>()
            .add_update_time(t.elapsed().as_secs_f32());
    }

    pub fn init() -> Egregoria {
        let mut world = World::default();
        let mut resources = Resources::default();

        info!("Seed is {}", RNG_SEED);

        // Basic resources init
        resources.insert(TimeInfo::default());
        resources.insert(CollisionWorld::new(50));
        resources.insert(KeyboardInfo::default());
        resources.insert(MouseInfo::default());
        resources.insert(InspectedEntity::default());
        resources.insert(FollowEntity::default());
        resources.insert(RenderStats::default());
        resources.insert(RandProvider::new(RNG_SEED));
        resources.insert(ParkingManagement::default());
        resources.insert(FrameLog::default());
        resources.insert(RunningScenario::default());
        resources.insert(ImmediateDraw::default());
        resources.insert(Souls::default());
        resources.insert(ParCommandBuffer::default());
        resources.insert(Tool::default());
        resources.insert(RunningScenario::default());
        resources.insert(Deleted::<Collider>::default());
        resources.insert(Deleted::<VehicleComponent>::default());
        resources.insert(ParCommandBuffer::default());

        // Systems state init
        let s = RoadBuildResource::new(&mut world);
        resources.insert(s);

        let s = RoadEditorResource::new(&mut world);
        resources.insert(s);

        let s = BulldozerResource::new(&mut world);
        resources.insert(s);

        // Dispatcher init
        let schedule = Schedule::builder()
            .add_system(vehicle_decision_system())
            .add_system(selectable_select_system())
            .add_system(selectable_cleanup_system())
            .add_system(roadbuild_system())
            .add_system(roadeditor_system())
            .add_system(bulldozer_system())
            .add_system(itinerary_update_system())
            .add_system(vehicle_cleanup_system())
            .add_system(pedestrian_decision_system())
            .add_system(run_scenario_system())
            .add_system(movable_system(MovableSystem::default()))
            .add_system(kinematics_apply_system())
            .add_system(coworld_synchronize_system())
            .add_system(coworld_maintain_system())
            .add_system(inspected_aura_system(InspectedAura::new(&mut world)))
            .build();

        Self {
            world,
            resources,
            schedule,
        }
    }

    pub fn comp<T: Component>(&self, e: Entity) -> Option<&T> {
        <&T>::query().get(&self.world, e).ok()
    }

    pub fn comp_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        <&mut T>::query().get_mut(&mut self.world, e).ok()
    }

    pub fn try_write<T: Resource>(&self) -> Option<impl DerefMut<Target = T> + '_> {
        self.resources.get_mut()
    }

    pub fn write<T: Resource>(&self) -> impl DerefMut<Target = T> + '_ {
        self.resources.get_mut().unwrap()
    }

    pub fn read<T: Resource>(&self) -> impl Deref<Target = T> + '_ {
        self.resources.get().unwrap()
    }

    pub fn insert<T: Resource>(&mut self, value: T) {
        self.resources.insert(value)
    }
}

pub fn load_from_disk(goria: &mut Egregoria) {
    let map: Map = saveload::load_or_default::<map_model::SerializedMap>("map").into();
    goria.insert(map);
    goria.insert(crate::saveload::load_or_default::<Gui>("gui"));
}

pub fn save_to_disk(goria: &mut Egregoria) {
    let _ = std::io::stdout().flush();
    crate::saveload::save(&*goria.read::<Gui>(), "gui");
    crate::saveload::save(&SerializedMap::from(&*goria.read::<Map>()), "map");
}
