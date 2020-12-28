#![allow(clippy::unreadable_literal)]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::too_many_arguments)]

use crate::economy::{Market, Wheat};
use crate::engine_interaction::{KeyboardInfo, MouseInfo, Movable, RenderStats, Selectable};
use crate::map_dynamic::{
    add_trees_system, itinerary_update_system, routing_update_system, BuildingInfos, Itinerary,
    ParkingManagement,
};
use crate::pedestrians::{pedestrian_decision_system, Pedestrian};
use crate::physics::systems::{
    coworld_maintain_system, coworld_synchronize_system, kinematics_apply_system,
};
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Kinematics};
use crate::rendering::assets::AssetRender;
use crate::rendering::immediate::{ImmediateDraw, ImmediateSound};
use crate::rendering::meshrender_component::MeshRender;
use crate::scenarios::scenario_runner::{run_scenario_system, RunningScenario};
use crate::souls::human::human_desires_system;
use crate::vehicles::systems::{
    vehicle_cleanup_system, vehicle_decision_system, vehicle_state_update_system,
};
use crate::vehicles::Vehicle;
use common::{GameTime, SECONDS_PER_DAY, SECONDS_PER_HOUR};
use geom::{Transform, Vec2};
use legion::storage::Component;
use legion::systems::Resource;
use legion::{any, Entity, IntoQuery, Registry, Resources, World};
use map_model::{Map, SerializedMap};
use pedestrians::Location;
use serde::{Deserialize, Serialize};
use souls::desire::{desire_home_system, desire_work_system};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};
use utils::frame_log::FrameLog;
use utils::par_command_buffer::Deleted;
pub use utils::par_command_buffer::ParCommandBuffer;
use utils::rand_provider::RandProvider;
use utils::scheduler::SeqSchedule;

#[macro_use]
extern crate imgui_inspect;

#[macro_use]
extern crate log as extern_log;

#[macro_use]
pub mod utils;

pub mod economy;
pub mod engine_interaction;
pub mod map_dynamic;
pub mod pedestrians;
pub mod physics;
pub mod rendering;
pub mod scenarios;
pub mod souls;
pub mod vehicles;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct SoulID(pub Entity);

debug_inspect_impl!(SoulID);

#[derive(Default)]
pub struct Egregoria {
    pub world: World,
    pub schedule: SeqSchedule,
    resources: Resources,
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
        goria.insert(GameTime::new(
            0.0,
            SECONDS_PER_DAY as f64 + 10.0 * SECONDS_PER_HOUR as f64,
        ));
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
        goria.insert(ImmediateSound::default());
        goria.insert(ParCommandBuffer::default());
        goria.insert(Deleted::<Collider>::default());
        goria.insert(Deleted::<Vehicle>::default());
        goria.insert(Market::<Wheat>::default());

        // Dispatcher init
        goria
            .schedule
            .add_system(vehicle_state_update_system())
            .add_system(vehicle_decision_system())
            .add_system(itinerary_update_system())
            .add_system(add_trees_system())
            .add_system(vehicle_cleanup_system())
            .add_system(pedestrian_decision_system())
            .add_system(run_scenario_system())
            .add_system(kinematics_apply_system())
            .add_system(coworld_synchronize_system())
            .add_system(routing_update_system())
            .add_system(desire_home_system())
            .add_system(desire_work_system())
            .add_system(human_desires_system())
            .add_system(coworld_maintain_system());

        goria
    }

    pub fn pos(&self, e: Entity) -> Option<Vec2> {
        self.comp::<Transform>(e).map(|x| x.position())
    }

    pub fn add_comp(&mut self, e: Entity, c: impl Component) {
        if self
            .world
            .entry(e)
            .map(move |mut e| e.add_component(c))
            .is_none()
        {
            log::error!("trying to add component to entity but it doesn't exist");
        }
    }

    pub fn comp<T: Component>(&self, e: Entity) -> Option<&T> {
        <&T>::query().get(&self.world, e).ok()
    }

    pub fn comp_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        <&mut T>::query().get_mut(&mut self.world, e).ok()
    }

    pub fn write_or_default<T: Resource + Default>(&mut self) -> impl DerefMut<Target = T> + '_ {
        self.resources.get_mut_or_insert_with(T::default)
    }

    pub fn try_write<T: Resource>(&self) -> Option<impl DerefMut<Target = T> + '_> {
        self.resources.get_mut()
    }

    pub fn write<T: Resource>(&self) -> impl DerefMut<Target = T> + '_ {
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
    ($r: expr; $($t: ty),+,) => {
        $(
            $r.register::<$t>(my_hash(stringify!($t)))
        );+
    };
}

pub struct NoSerialize;

macro_rules! mk_save {
    ($($res: ty,)*) => {
        fn registry() -> Registry<u64> {
            let mut registry = Registry::default();
            register!(registry; Transform,
              AssetRender,
              Kinematics,
              Selectable,
              Movable,
              Vehicle,
              Pedestrian,
              Itinerary,
              Collider,
              MeshRender,
              Location,
              $($res,)*
            );
            registry
        }

        pub fn load_from_disk(goria: &mut Egregoria) {
            let registry = registry();

            let _ = common::saveload::load_seed("world", registry.as_deserialize()).map(|mut w: World| {
                log::info!("successfully loaded world with {} entities", w.len());
                goria.world.move_from(&mut w, &any());
            });

            $(
            extract_resource::<$res>(goria);
            )*

            goria.insert::<Map>(
                common::saveload::load::<map_model::SerializedMap>("map")
                    .map(|x| x.into())
                    .unwrap_or_default(),
            );
        }

        pub fn save_to_disk(goria: &mut Egregoria) {
            let registry = registry();

            let to_remove = vec![
                $(
                insert_resource::<$res>(goria),
                )*
            ];

            let s = goria
                .world
                .as_serializable(!legion::query::component::<NoSerialize>(), &registry);

            common::saveload::save(&s, "world");
            common::saveload::save(&SerializedMap::from(&*goria.read::<Map>()), "map");

            for ent in to_remove {
                goria.world.remove(ent);
            }
        }
    }
}

mk_save!(CollisionWorld, ParkingManagement, BuildingInfos,);

fn extract_resource<T: Resource + Clone + Sync + Send>(goria: &mut Egregoria) {
    let (ent, res): (&Entity, &T) = match <(Entity, &T)>::query().iter(&goria.world).next() {
        Some(x) => x,
        None => {
            info!("Resource {} was not serialized", std::any::type_name::<T>());
            return;
        }
    };

    let (ent, res) = (*ent, res.clone());

    goria.world.remove(ent);

    goria.resources.insert(res);
}

fn insert_resource<T: Resource + Clone + Sync + Send>(goria: &mut Egregoria) -> Entity {
    let res: T = goria.read::<T>().clone();
    goria.world.push((res,))
}
