#![allow(clippy::unreadable_literal)]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::upper_case_acronyms)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::unwrap_used)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::economy::{Bought, Sold, Workers};
use crate::engine_interaction::{Selectable, WorldCommands};
use crate::map_dynamic::{Itinerary, Router};
use crate::pedestrians::Pedestrian;
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Kinematics};
use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use crate::souls::add_souls_to_empty_buildings;
use crate::souls::desire::{BuyFood, Home, Work};
use crate::souls::goods_company::GoodsCompany;
use crate::vehicles::Vehicle;
use atomic_refcell::{AtomicRef, AtomicRefMut};
use common::saveload::Encoder;
use common::FastMap;
use geom::{Transform, Vec2};
use legion::serialize::{Canon, CustomEntitySerializer};
use legion::storage::Component;
use legion::systems::{ParallelRunnable, Resource};
use legion::{Entity, IntoQuery, Registry, Resources, World};
use map_model::Map;
use pedestrians::Location;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::num::NonZeroU64;
use std::sync::atomic::AtomicU64;
use std::time::{Duration, Instant};
pub use utils::par_command_buffer::ParCommandBuffer;
use utils::rand_provider::RandProvider;
use utils::scheduler::SeqSchedule;
use utils::time::{GameTime, SECONDS_PER_DAY, SECONDS_PER_HOUR};

macro_rules! register_system {
    ($f: ident) => {
        inventory::submit! {
            paste::paste! {
                $crate::GSystem::new(Box::new(|| Box::new([<$f _system >]())))
            }
        }
    };
}

macro_rules! init_func {
    ($f: expr) => {
        inventory::submit! {
            $crate::InitFunc {
                f: Box::new($f),
            }
        }
    };
}

macro_rules! register_resource {
    ($t: ty, $name: expr) => {
        init_func!(|goria| {
            goria.insert(<$t>::default());
        });
        inventory::submit! {
            $crate::SaveLoadFunc {
                name: $name,
                save: Box::new(|goria| {
                     <common::saveload::Bincode as common::saveload::Encoder>::encode(&*goria.read::<$t>()).unwrap()
                }),
                load: Box::new(|goria, v| {
                    if let Some(v) = v {
                        if let Ok(res) = <common::saveload::Bincode as common::saveload::Encoder>::decode::<$t>(&v) {
                            goria.insert(res);
                        }
                    }
                })
            }
        }
    };
    ($t: ty, $name: expr, $init: expr) => {
        init_func!(|goria| {
            goria.insert($init);
        });
        inventory::submit! {
            $crate::SaveLoadFunc {
                name: $name,
                save: Box::new(|goria| {
                     <common::saveload::Bincode as common::saveload::Encoder>::encode(&*goria.read::<$t>()).unwrap()
                }),
                load: Box::new(|goria, v| {
                    if let Some(v) = v {
                        if let Ok(res) = <common::saveload::Bincode as common::saveload::Encoder>::decode::<$t>(&v) {
                            goria.insert(res);
                        }
                    }
                })
            }
        }
    };
}

macro_rules! register_resource_noserialize {
    ($t: ty) => {
        init_func!(|goria| {
            goria.insert(<$t>::default());
        });
    };
}

register_resource!(Map, "map");

register_resource!(
    GameTime,
    "game_time",
    GameTime::new(0.0, SECONDS_PER_DAY as f64 + 10.0 * SECONDS_PER_HOUR as f64,)
);

register_resource!(CollisionWorld, "coworld", CollisionWorld::new(100));
register_resource!(RandProvider, "randprovider", RandProvider::new(RNG_SEED));

#[macro_use]
extern crate common;

#[macro_use]
extern crate imgui_inspect;

#[macro_use]
extern crate log as extern_log;

pub mod economy;
pub mod engine_interaction;
pub mod map_dynamic;
pub mod pedestrians;
pub mod physics;
pub mod rendering;
pub mod souls;
mod tests;
pub mod utils;
pub mod vehicles;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[repr(transparent)]
pub struct SoulID(pub Entity);

impl PartialOrd for SoulID {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let sel: NonZeroU64 = unsafe { std::mem::transmute(self.0) };
        let other: NonZeroU64 = unsafe { std::mem::transmute(other.0) };
        sel.partial_cmp(&other)
    }
}

impl Ord for SoulID {
    fn cmp(&self, other: &Self) -> Ordering {
        let sel: NonZeroU64 = unsafe { std::mem::transmute(self.0) };
        let other: NonZeroU64 = unsafe { std::mem::transmute(other.0) };
        sel.cmp(&other)
    }
}

debug_inspect_impl!(SoulID);

pub struct Egregoria {
    pub(crate) world: World,
    resources: Resources,
    tick: u32,
}

pub(crate) struct SaveLoadFunc {
    pub name: &'static str,
    pub save: Box<dyn Fn(&Egregoria) -> Vec<u8> + 'static>,
    pub load: Box<dyn Fn(&mut Egregoria, Option<Vec<u8>>) + 'static>,
}
inventory::collect!(SaveLoadFunc);

pub(crate) struct InitFunc {
    pub f: Box<dyn Fn(&mut Egregoria) + 'static>,
}
inventory::collect!(InitFunc);

pub(crate) struct GSystem {
    s: Box<dyn Fn() -> Box<dyn ParallelRunnable + 'static>>,
}

impl GSystem {
    pub fn new(s: Box<dyn Fn() -> Box<dyn ParallelRunnable + 'static>>) -> Self {
        Self { s }
    }
}
inventory::collect!(GSystem);

/// Safety: Resources must be Send+Sync.
/// Guaranteed by Egregoria::insert.
/// World is Send+Sync and SeqSchedule too
unsafe impl Sync for Egregoria {}

const RNG_SEED: u64 = 123;

impl Egregoria {
    pub fn schedule() -> SeqSchedule {
        let mut schedule = SeqSchedule::default();
        for s in inventory::iter::<GSystem> {
            let s = (s.s)();
            schedule.add_system(s);
        }
        schedule
    }

    pub fn empty() -> Egregoria {
        let mut goria = Egregoria {
            world: Default::default(),
            resources: Default::default(),
            tick: 0,
        };

        info!("Seed is {}", RNG_SEED);

        for s in inventory::iter::<InitFunc> {
            (s.f)(&mut goria);
        }

        goria
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn tick(&mut self, game_schedule: &mut SeqSchedule, commands: &WorldCommands) -> Duration {
        self.tick += 1;
        const WORLD_TICK_DT: f32 = 0.05;

        let t = Instant::now();

        {
            let mut time = self.write::<GameTime>();
            *time = GameTime::new(WORLD_TICK_DT, time.timestamp + WORLD_TICK_DT as f64);
        }

        for command in &commands.commands {
            command.apply(self);
        }

        game_schedule.execute(self);
        add_souls_to_empty_buildings(self);
        t.elapsed()
    }

    pub fn get_tick(&self) -> u32 {
        self.tick
    }

    pub fn hashes(&self) -> BTreeMap<String, u64> {
        fn hash(x: &[u8]) -> u64 {
            let mut h = DefaultHasher::new();
            h.write(&x);
            h.finish()
        }
        let serworld = unwrap_ret!(
            SerPreparedEgregoria::try_from(self).ok(),
            Default::default()
        );

        let mut hashes = BTreeMap::new();
        hashes.insert("tick".to_string(), serworld.tick as u64);
        hashes.insert("world".to_string(), hash(&*serworld.world));
        for (name, v) in serworld.res {
            hashes.insert(name, hash(&*v));
        }

        hashes
    }

    pub fn load_from_disk(save_name: &'static str) -> Option<Self> {
        let ser: SerPreparedEgregoria = common::saveload::CompressedBincode::load(save_name)?;
        Self::try_from(ser).ok()
    }

    pub fn save_to_disk(&self, save_name: &'static str) {
        let ser = unwrap_retlog!(
            SerPreparedEgregoria::try_from(self).ok(),
            "failed saving, couldn't serialize"
        );
        common::saveload::CompressedBincode::save(&ser, save_name);
    }

    pub fn pos(&self, e: Entity) -> Option<Vec2> {
        self.comp::<Transform>(e).map(|x| x.position())
    }

    pub(crate) fn add_comp(&mut self, e: Entity, c: impl Component) {
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

    pub(crate) fn comp_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        <&mut T>::query().get_mut(&mut self.world, e).ok()
    }

    #[allow(dead_code)]
    pub(crate) fn try_write<T: Resource>(&self) -> Option<AtomicRefMut<T>> {
        self.resources.get_mut()
    }

    pub(crate) fn write<T: Resource>(&self) -> AtomicRefMut<T> {
        self.resources
            .get_mut()
            .unwrap_or_else(|| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn read<T: Resource>(&self) -> AtomicRef<T> {
        self.resources
            .get()
            .unwrap_or_else(|| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn map(&self) -> AtomicRef<Map> {
        self.resources.get().unwrap()
    }

    pub(crate) fn map_mut(&self) -> AtomicRefMut<Map> {
        self.resources.get_mut().unwrap()
    }

    pub fn insert<T: Resource>(&mut self, res: T) {
        self.resources.insert(res)
    }
}

impl TryFrom<&Egregoria> for SerPreparedEgregoria {
    type Error = ();

    fn try_from(goria: &Egregoria) -> Result<Self, ()> {
        let registry = registry();

        let entity_serializer = IdSer::default();
        let s = goria.world.as_serializable(
            !legion::query::component::<NoSerialize>(),
            &registry,
            &entity_serializer,
        );

        let world = common::saveload::Bincode::encode(&s).map_err(|_| ())?;

        let mut m: FastMap<String, Vec<u8>> = FastMap::default();

        legion::serialize::set_entity_serializer(&entity_serializer, || {
            for l in inventory::iter::<SaveLoadFunc> {
                let v = (l.save)(goria);
                m.insert(l.name.to_string(), v);
            }
        });

        Ok(SerPreparedEgregoria {
            world,
            res: m,
            tick: goria.tick,
        })
    }
}

impl TryFrom<SerPreparedEgregoria> for Egregoria {
    type Error = std::io::Error;

    fn try_from(mut ser: SerPreparedEgregoria) -> Result<Self, Self::Error> {
        let mut goria = Self::empty();
        goria.tick = ser.tick;
        let registry = registry();

        let entity_serializer = IdSer::default();

        let w: World = common::saveload::Bincode::decode_seed(
            registry.as_deserialize(&entity_serializer),
            &ser.world,
        )?;

        goria.world = w;

        legion::serialize::set_entity_serializer(&entity_serializer, || {
            for l in inventory::iter::<SaveLoadFunc> {
                (l.load)(&mut goria, ser.res.remove(l.name));
            }
        });

        let max_deser = entity_serializer
            .max_deser
            .load(std::sync::atomic::Ordering::SeqCst);

        const BLOCK_SIZE: u64 = 16;
        let mut p = BLOCK_SIZE;
        while p <= max_deser {
            // up block size
            let c = Canon::default();
            c.canonize_name(&[0; 16]);
            p += BLOCK_SIZE
        }

        Ok(goria)
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerPreparedEgregoria {
    world: Vec<u8>,
    res: FastMap<String, Vec<u8>>,
    tick: u32,
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

fn registry() -> Registry<u64> {
    let mut registry = Registry::default();
    register!(registry;
      AssetRender,
      Bought,
      BuyFood,
      Collider,
      GoodsCompany,
      Home,
      Itinerary,
      Kinematics,
      Location,
      MeshRender,
      Pedestrian,
      Router,
      Selectable,
      Sold,
      Transform,
      Vehicle,
      Work,
      Workers,
    );
    registry
}

pub fn ent_id(e: Entity) -> u64 {
    unsafe { std::mem::transmute(e) }
}

pub fn ent_from_id(x: u64) -> Entity {
    if x == 0 {
        panic!("x is zero");
    }
    unsafe { std::mem::transmute(x) }
}

#[derive(Default)]
pub struct IdSer {
    max_deser: AtomicU64,
}

impl CustomEntitySerializer for IdSer {
    type SerializedID = u64;

    fn to_serialized(&self, entity: Entity) -> Self::SerializedID {
        ent_id(entity)
    }

    fn from_serialized(&self, serialized: u64) -> Entity {
        use std::sync::atomic::Ordering::SeqCst;
        loop {
            let v = self.max_deser.load(SeqCst);
            if serialized > v {
                if self
                    .max_deser
                    .compare_exchange(v, serialized, SeqCst, SeqCst)
                    .is_ok()
                {
                    break;
                }
            } else {
                break;
            }
        }
        ent_from_id(serialized)
    }
}
