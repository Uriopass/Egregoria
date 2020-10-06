#![allow(clippy::unreadable_literal)]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::too_many_arguments)]

use crate::engine_interaction::{
    KeyboardInfo, MouseInfo, Movable, RenderStats, Selectable, TimeWarp,
};
use crate::map_dynamic::{itinerary_update_system, BuildingInfos, Itinerary, ParkingManagement};
use crate::pedestrians::{pedestrian_decision_system, pedestrian_synchro_system, Pedestrian};
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
use common::GameTime;
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
pub mod economy;
pub mod engine_interaction;
pub mod map_dynamic;
pub mod pedestrians;
pub mod physics;
pub mod rendering;
pub mod scenarios;
pub mod vehicles;

use crate::economy::Market;
use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use geom::{Transform, Vec2};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use utils::par_command_buffer::Deleted;
use utils::scheduler::SeqSchedule;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SoulID(pub usize);

#[derive(Default)]
pub struct Egregoria {
    pub world: World,
    pub schedule: SeqSchedule,
    resources: Resources,
    read_only: bool,
}

/// Safety: Resources must be Send+Sync.
/// Guaranteed by Egregoria::insert.
/// World is Send+Sync and SeqSchedule too
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
            .add_value(t.elapsed().as_secs_f32());
    }

    pub fn init() -> Egregoria {
        let mut goria = Egregoria::default();

        info!("Seed is {}", RNG_SEED);

        // Basic assets init
        goria.insert(GameTime::new(0.0, 0.0));
        goria.insert(CollisionWorld::new(100));
        goria.insert(KeyboardInfo::default());
        goria.insert(MouseInfo::default());
        goria.insert(RenderStats::default());
        goria.insert(RandProvider::new(RNG_SEED));
        goria.insert(ParkingManagement::default());
        goria.insert(BuildingInfos::default());
        goria.insert(FrameLog::default());
        goria.insert(RunningScenario::default());
        goria.insert(ImmediateDraw::default());
        goria.insert(ParCommandBuffer::default());
        goria.insert(Deleted::<Collider>::default());
        goria.insert(Deleted::<Vehicle>::default());
        goria.insert(Market::default());
        goria.insert(TimeWarp::default());

        // Dispatcher init
        goria
            .schedule
            .add_system(vehicle_state_update_system())
            .add_system(vehicle_decision_system())
            .add_system(itinerary_update_system())
            .add_system(vehicle_cleanup_system())
            .add_system(pedestrian_decision_system())
            .add_system(pedestrian_synchro_system())
            .add_system(run_scenario_system())
            .add_system(kinematics_apply_system())
            .add_system(coworld_synchronize_system())
            .add_system(coworld_maintain_system());

        goria
    }

    pub fn pos(&self, e: Entity) -> Option<Vec2> {
        self.comp::<Transform>(e).map(|x| x.position())
    }

    pub fn comp<T: Component>(&self, e: Entity) -> Option<&T> {
        <&T>::query().get(&self.world, e).ok()
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    pub fn comp_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        <&mut T>::query().get_mut(&mut self.world, e).ok()
    }

    pub fn write_or_default<T: Resource + Default>(&mut self) -> impl DerefMut<Target = T> + '_ {
        self.resources.get_mut_or_insert_with(T::default)
    }

    pub fn try_write<T: Resource>(&self) -> Option<impl DerefMut<Target = T> + '_> {
        debug_assert!(!self.read_only);
        self.resources.get_mut()
    }

    pub fn write<T: Resource>(&self) -> impl DerefMut<Target = T> + '_ {
        debug_assert!(!self.read_only);
        self.resources
            .get_mut()
            .unwrap_or_else(|| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn read<T: Resource>(&self) -> impl Deref<Target = T> + '_ {
        self.resources
            .get()
            .unwrap_or_else(|| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
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
    goria.insert::<Map>(
        common::saveload::load::<map_model::SerializedMap>("map")
            .map(|x| x.into())
            .unwrap_or_default(),
    );

    goria.insert(common::saveload::load_or_default::<ParkingManagement>(
        "parking",
    ));

    let registry = registry();

    let _ = common::saveload::load_seed("world", registry.as_deserialize()).map(|mut w: World| {
        log::info!("successfully loaded world with {} entities", w.len());
        goria.world.move_from(&mut w, &any());
    });

    deserialize_colliders(goria);
}

pub fn save_to_disk(goria: &mut Egregoria) {
    let _ = std::io::stdout().flush();
    common::saveload::save(&*goria.read::<ParkingManagement>(), "parking");
    common::saveload::save(&SerializedMap::from(&*goria.read::<Map>()), "map");

    let registry = registry();

    let s = goria
        .world
        .as_serializable(!legion::query::component::<NoSerialize>(), &registry);
    common::saveload::save(&s, "world");

    serialize_colliders(goria);
}
