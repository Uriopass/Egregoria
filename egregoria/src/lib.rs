#![allow(clippy::unreadable_literal)]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::too_many_arguments)]

use crate::engine_interaction::{
    KeyboardInfo, MouseInfo, Movable, RenderStats, Selectable, TimeInfo,
};
use crate::map_dynamic::{itinerary_update_system, BuildingInfos, Itinerary, ParkingManagement};
use crate::pedestrians::{pedestrian_decision_system, Pedestrian};
use crate::physics::systems::{
    coworld_maintain_system, coworld_synchronize_system, kinematics_apply_system,
};
use crate::physics::{deserialize_colliders, serialize_colliders, CollisionWorld};
use crate::physics::{Collider, Kinematics};
use crate::rendering::immediate::ImmediateDraw;
use crate::scenarios::scenario_runner::{run_scenario_system, RunningScenario};
use crate::vehicles::systems::{
    vehicle_cleanup_system, vehicle_decision_system, vehicle_state_update_system,
};
use crate::vehicles::Vehicle;
pub use imgui;
use legion::storage::Component;
use legion::systems::Resource;
use legion::{any, Entity, IntoQuery, Registry, Resources, World};
use map_model::{Map, SerializedMap};
use std::io::Write;
use std::ops::{Deref, DerefMut};
use utils::frame_log::FrameLog;
pub use utils::par_command_buffer::ParCommandBuffer;
use utils::rand_provider::RandProvider;

#[macro_use]
extern crate imgui_inspect;

#[macro_use]
extern crate log as extern_log;

#[macro_use]
pub mod utils;

pub mod api;
pub mod engine_interaction;
pub mod map_dynamic;
pub mod pedestrians;
pub mod physics;
pub mod rendering;
pub mod scenarios;
pub mod vehicles;

use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use geom::{Transform, Vec2};
pub use legion;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use utils::par_command_buffer::Deleted;
use utils::saveload;
use utils::scheduler::SeqSchedule;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SoulID(pub u64);

pub struct Egregoria {
    pub world: World,
    pub schedule: SeqSchedule,
    resources: Resources,
}

// Safety: Resources must be Send+Sync
// World is Send+Sync and SeqSchedule too
unsafe impl Sync for Egregoria {}

const RNG_SEED: u64 = 123;

impl Egregoria {
    pub fn run(&mut self) {
        self.read::<FrameLog>().clear();
        let t = std::time::Instant::now();
        self.schedule.execute(&mut self.world, &mut self.resources);
        ParCommandBuffer::apply(self);
        self.write::<RenderStats>()
            .world_update
            .add_time(t.elapsed().as_secs_f32());
    }

    pub fn init() -> Egregoria {
        let world = World::default();
        let mut resources = Resources::default();

        info!("Seed is {}", RNG_SEED);

        // Basic assets init
        resources.insert(TimeInfo::default());
        resources.insert(CollisionWorld::new(50));
        resources.insert(KeyboardInfo::default());
        resources.insert(MouseInfo::default());
        resources.insert(RenderStats::default());
        resources.insert(RandProvider::new(RNG_SEED));
        resources.insert(ParkingManagement::default());
        resources.insert(BuildingInfos::default());
        resources.insert(FrameLog::default());
        resources.insert(RunningScenario::default());
        resources.insert(ImmediateDraw::default());
        resources.insert(ParCommandBuffer::default());
        resources.insert(Deleted::<Collider>::default());
        resources.insert(Deleted::<Vehicle>::default());

        // Dispatcher init
        let mut schedule = SeqSchedule::default();
        schedule
            .add_system(vehicle_state_update_system())
            .add_system(vehicle_decision_system())
            .add_system(itinerary_update_system())
            .add_system(vehicle_cleanup_system())
            .add_system(pedestrian_decision_system())
            .add_system(run_scenario_system())
            .add_system(kinematics_apply_system())
            .add_system(coworld_synchronize_system())
            .add_system(coworld_maintain_system());

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

    pub fn position(&self, e: Entity) -> Option<Vec2> {
        self.comp::<Transform>(e).map(|x| x.position())
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

    pub fn insert<T: Resource + Send + Sync>(&mut self, res: T) {
        self.resources.insert(res)
    }
}

fn my_hash<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}

macro_rules! register {
    ($r: expr, $t: ty) => {
        $r.register::<$t>(my_hash(stringify!($t)))
    };
}

fn registry() -> Registry<u64> {
    let mut registry = Registry::default();
    register!(registry, Transform);
    register!(registry, AssetRender);
    register!(registry, Kinematics);
    register!(registry, Selectable);
    register!(registry, Movable);
    register!(registry, Vehicle);
    register!(registry, Pedestrian);
    register!(registry, Itinerary);
    register!(registry, Collider);
    register!(registry, MeshRender);
    registry
}

pub struct NoSerialize;

pub fn load_from_disk(goria: &mut Egregoria) {
    goria.insert(Map::from(saveload::load_or_default::<
        map_model::SerializedMap,
    >("map")));

    goria.insert(crate::saveload::load_or_default::<ParkingManagement>(
        "parking",
    ));

    let registry = registry();

    let _ = saveload::load_seed("world", registry.as_deserialize()).map(|mut w: World| {
        log::info!("successfully loaded world with {} entities", w.len());
        goria.world.move_from(&mut w, &any());
    });

    deserialize_colliders(goria);
}

pub fn save_to_disk(goria: &mut Egregoria) {
    let _ = std::io::stdout().flush();
    crate::saveload::save(&*goria.read::<ParkingManagement>(), "parking");
    crate::saveload::save(&SerializedMap::from(&*goria.read::<Map>()), "map");

    let registry = registry();

    let s = goria
        .world
        .as_serializable(!legion::query::component::<NoSerialize>(), &registry);
    crate::saveload::save(&s, "world");

    serialize_colliders(goria);
}
