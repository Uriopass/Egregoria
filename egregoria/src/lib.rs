#![allow(clippy::unreadable_literal)]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::too_many_arguments)]

use crate::economy::{Bought, Sold, Workers};
use crate::engine_interaction::{RenderStats, Selectable};
use crate::map_dynamic::{Itinerary, Router};
use crate::pedestrians::Pedestrian;
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Kinematics};
use crate::rendering::assets::AssetRender;
use crate::rendering::meshrender_component::MeshRender;
use crate::souls::desire::{BuyFood, Desire, Home, Work};
use crate::vehicles::Vehicle;
use atomic_refcell::{AtomicRef, AtomicRefMut};
use common::{GameTime, SECONDS_PER_DAY, SECONDS_PER_HOUR};
use geom::{Transform, Vec2};
use legion::serialize::Canon;
use legion::storage::Component;
use legion::systems::{ParallelRunnable, Resource};
use legion::{any, Entity, IntoQuery, Registry, Resources, World};
use map_model::{Map, SerializedMap};
use pedestrians::Location;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use utils::frame_log::FrameLog;
use utils::par_command_buffer::Deleted;
pub use utils::par_command_buffer::ParCommandBuffer;
use utils::rand_provider::RandProvider;
use utils::scheduler::SeqSchedule;

#[macro_export]
macro_rules! register_system {
    ($f: ident) => {
        inventory::submit! {
            paste::paste! {
                $crate::GSystem::new(std::cell::RefCell::new(Some(Box::new([<$f _system >]()))))
            }
        }
    };
}

#[macro_export]
macro_rules! init_func {
    ($f: expr) => {
        inventory::submit! {
            $crate::InitFunc {
                f: Box::new($f),
            }
        }
    };
}

#[macro_export]
macro_rules! register_resource {
    ($t: ty, $name: expr) => {
        init_func!(|goria| {
            goria.insert(<$t>::default());
        });
        inventory::submit! {
            $crate::SaveLoadFunc {
                save: Box::new(|goria| {
                     common::saveload::save(&*goria.read::<$t>(), $name);
                }),
                load: Box::new(|goria| {
                    if let Some(res) = common::saveload::load::<$t>($name) {
                        goria.insert(res);
                    }
                })
            }
        }
    };
}

#[macro_export]
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
pub mod scenarios;
pub mod souls;
pub mod utils;
pub mod vehicles;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[repr(transparent)]
pub struct SoulID(pub Entity);

debug_inspect_impl!(SoulID);

#[derive(Default)]
pub struct Egregoria {
    pub world: World,
    pub schedule: SeqSchedule,
    resources: Resources,
}

pub struct SaveLoadFunc {
    pub save: Box<dyn Fn(&mut Egregoria) + 'static>,
    pub load: Box<dyn Fn(&mut Egregoria) + 'static>,
}
inventory::collect!(SaveLoadFunc);

pub struct InitFunc {
    pub f: Box<dyn Fn(&mut Egregoria) + 'static>,
}

inventory::collect!(InitFunc);

pub struct GSystem {
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
        goria.insert(RandProvider::new(RNG_SEED));
        goria.insert(Deleted::<Collider>::default());
        goria.insert(Deleted::<Vehicle>::default());

        for s in inventory::iter::<InitFunc> {
            (s.f)(&mut goria);
        }

        for s in inventory::iter::<GSystem> {
            let s = s.s.borrow_mut().take().unwrap();
            goria.schedule.add_system(s);
        }

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

    pub fn write_or_default<T: Resource + Default>(&mut self) -> AtomicRefMut<T> {
        self.resources.get_mut_or_insert_with(T::default)
    }

    pub fn try_write<T: Resource>(&self) -> Option<AtomicRefMut<T>> {
        self.resources.get_mut()
    }

    pub fn write<T: Resource>(&self) -> AtomicRefMut<T> {
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

pub fn save_to_disk(goria: &mut Egregoria) {
    let registry = registry();

    let entity_serializer = Canon::default();
    let s = goria.world.as_serializable(
        !legion::query::component::<NoSerialize>(),
        &registry,
        &entity_serializer,
    );

    common::saveload::save(&s, "world");
    common::saveload::save(&SerializedMap::from(&*goria.read::<Map>()), "map");

    legion::serialize::set_entity_serializer(&entity_serializer, || {
        for l in inventory::iter::<SaveLoadFunc> {
            (l.save)(goria);
        }
    });
}

pub fn load_from_disk(goria: &mut Egregoria) {
    let registry = registry();

    let entity_serializer = Canon::default();
    let _ = common::saveload::load_seed("world", registry.as_deserialize(&entity_serializer)).map(
        |mut w: World| {
            log::info!("successfully loaded world with {} entities", w.len());
            goria.world.move_from(&mut w, &any());
        },
    );

    legion::serialize::set_entity_serializer(&entity_serializer, || {
        for l in inventory::iter::<SaveLoadFunc> {
            (l.load)(goria);
        }
    });

    goria.insert::<Map>(
        common::saveload::load::<map_model::SerializedMap>("map")
            .map(|x| x.into())
            .unwrap_or_default(),
    );
}
