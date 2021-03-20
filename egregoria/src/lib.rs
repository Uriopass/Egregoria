#![allow(clippy::unreadable_literal)]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::too_many_arguments)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use atomic_refcell::{AtomicRef, AtomicRefMut};
use legion::serialize::Canon;
use legion::storage::Component;
use legion::systems::{ParallelRunnable, Resource};
use legion::{any, Entity, IntoQuery, Registry, Resources, World};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use common::{GameTime, SECONDS_PER_DAY, SECONDS_PER_HOUR};
use geom::{Transform, Vec2};
use map_model::{Map, SerializedMap};
use pedestrians::Location;
use utils::par_command_buffer::Deleted;
pub use utils::par_command_buffer::ParCommandBuffer;
use utils::rand_provider::RandProvider;
use utils::scheduler::SeqSchedule;

use crate::economy::{Bought, Sold, Workers};
use crate::engine_interaction::Selectable;
use crate::map_dynamic::{Itinerary, Router};
use crate::pedestrians::Pedestrian;
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Kinematics};
use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use crate::souls::add_souls_to_empty_buildings;
use crate::souls::desire::{BuyFood, Desire, Home, Work};
use crate::vehicles::Vehicle;
use serde::de::Error;
use std::collections::HashMap;

macro_rules! register_system {
    ($f: ident) => {
        inventory::submit! {
            paste::paste! {
                $crate::GSystem::new(std::cell::RefCell::new(Some(Box::new([<$f _system >]()))))
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
                     common::saveload::encode(&*goria.read::<$t>()).unwrap()
                }),
                load: Box::new(|goria, v| {
                    if let Some(v) = v {
                        if let Some(res) = common::saveload::decode::<$t>(&v) {
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
pub mod utils;
pub mod vehicles;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[repr(transparent)]
pub struct SoulID(pub Entity);

debug_inspect_impl!(SoulID);

pub struct Egregoria {
    pub(crate) world: World,
    resources: Resources,
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
    s: std::cell::RefCell<Option<Box<dyn ParallelRunnable + 'static>>>,
}

impl GSystem {
    pub fn new(s: std::cell::RefCell<Option<Box<dyn ParallelRunnable + 'static>>>) -> Self {
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
            let s = s.s.borrow_mut().take().unwrap();
            schedule.add_system(s);
        }
        schedule
    }

    pub fn empty() -> Egregoria {
        let mut goria = Egregoria {
            world: Default::default(),
            resources: Default::default(),
        };

        info!("Seed is {}", RNG_SEED);

        // Basic assets init
        goria.insert(GameTime::new(
            0.0,
            SECONDS_PER_DAY as f64 + 10.0 * SECONDS_PER_HOUR as f64,
        ));
        goria.insert(CollisionWorld::new(100));
        goria.insert(RandProvider::new(RNG_SEED));
        goria.insert(Deleted::<Collider>::default());
        goria.insert(Deleted::<Vehicle>::default());
        goria.insert(Map::empty());

        for s in inventory::iter::<InitFunc> {
            (s.f)(&mut goria);
        }

        goria
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn tick(&mut self, dt: f64, game_schedule: &mut SeqSchedule) {
        {
            let mut time = self.write::<GameTime>();
            *time = GameTime::new(dt as f32, time.timestamp + dt);
        }

        game_schedule.execute(self);
        add_souls_to_empty_buildings(self);
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

    pub fn insert<T: Resource + Send + Sync>(&mut self, res: T) {
        self.resources.insert(res)
    }
}

impl Serialize for Egregoria {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let registry = registry();

        let entity_serializer = Canon::default();
        let s = self.world.as_serializable(
            !legion::query::component::<NoSerialize>(),
            &registry,
            &entity_serializer,
        );

        let world = common::saveload::encode(&s).unwrap();

        let mut m: HashMap<String, Vec<u8>> = HashMap::new();

        legion::serialize::set_entity_serializer(&entity_serializer, || {
            for l in inventory::iter::<SaveLoadFunc> {
                let v = (l.save)(self);
                m.insert(l.name.to_string(), v);
            }
        });

        let ser = SerializedWorld {
            world,
            map: SerializedMap::from(&*self.read::<Map>()),
            res: m,
        };

        <SerializedWorld as Serialize>::serialize(&ser, serializer)
    }
}

impl<'de> Deserialize<'de> for Egregoria {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let mut ser: SerializedWorld = <SerializedWorld as Deserialize>::deserialize(deserializer)?;

        let mut goria = Self::empty();
        let registry = registry();

        let entity_serializer = Canon::default();

        let mut w: World =
            common::saveload::decode_seed(registry.as_deserialize(&entity_serializer), &ser.world)
                .ok_or_else(|| <D as Deserializer>::Error::custom("error deserializing world"))?;

        goria.world.move_from(&mut w, &any());

        legion::serialize::set_entity_serializer(&entity_serializer, || {
            for l in inventory::iter::<SaveLoadFunc> {
                (l.load)(&mut goria, ser.res.remove(l.name));
            }
        });

        goria.insert::<Map>(ser.map.into());
        Ok(goria)
    }
}

#[derive(Serialize, Deserialize)]
struct SerializedWorld {
    world: Vec<u8>,
    map: SerializedMap,
    res: HashMap<String, Vec<u8>>,
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
      Transform,
      AssetRender,
      Kinematics,
      Selectable,
      Vehicle,
      Pedestrian,
      Itinerary,
      Collider,
      MeshRender,
      Location,
      Desire<Home>,
      Desire<BuyFood>,
      Desire<Work>,
      Bought,
      Sold,
      Workers,
      Router,
    );
    registry
}
