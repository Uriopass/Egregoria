use crate::economy::{Bought, Sold, Workers};
use crate::engine_interaction::{Selectable, WorldCommands};
use crate::map::{BuildingGen, BuildingKind, LanePatternBuilder, Map, StraightRoadGen, Terrain};
use crate::map_dynamic::{Itinerary, ItineraryFollower, ItineraryLeader, Router};
use crate::pedestrians::Pedestrian;
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Kinematics};
use crate::souls::add_souls_to_empty_buildings;
use crate::souls::desire::{BuyFood, Home, Work};
use crate::souls::goods_company::{GoodsCompany, GoodsCompanyRegistry};
use crate::souls::human::HumanDecision;
use crate::vehicles::trains::{Locomotive, LocomotiveReservation, RandomLocomotive};
use crate::vehicles::Vehicle;
use common::saveload::Encoder;
use geom::{vec3, Transform, Vec2, Vec3, OBB};
use hecs::{Component, Entity, World};
use pedestrians::Location;
use resources::{Ref, RefMut, Resource, Resources};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
use utils::rand_provider::RandProvider;
use utils::scheduler::SeqSchedule;
use utils::time::{GameTime, SECONDS_PER_DAY, SECONDS_PER_HOUR};

#[macro_use]
extern crate common;

#[macro_use]
extern crate egui_inspect;

#[macro_use]
extern crate log as extern_log;

pub mod economy;
pub mod engine_interaction;
pub mod init;
pub mod map;
pub mod map_dynamic;
pub mod pedestrians;
pub mod physics;
pub mod souls;
mod tests;
pub mod utils;
pub mod vehicles;

use crate::init::{GSYSTEMS, INIT_FUNCS, SAVELOAD_FUNCS};
use crate::souls::fret_station::FreightStation;
use crate::utils::scheduler::RunnableSystem;
use crate::utils::time::Tick;
use crate::vehicles::trains::RailWagon;
use common::FastMap;
use serde::de::Error;
pub use utils::par_command_buffer::ParCommandBuffer;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[repr(transparent)]
pub struct SoulID(pub Entity);

debug_inspect_impl!(SoulID);

pub struct Egregoria {
    pub(crate) world: World,
    resources: Resources,
}

/// Safety: Resources must be Send+Sync.
/// Guaranteed by `Egregoria::insert`.
/// World is Send+Sync and `SeqSchedule` too
unsafe impl Sync for Egregoria {}

const RNG_SEED: u64 = 123;
const VERSION: &str = include_str!("../../VERSION");

impl Egregoria {
    pub fn schedule() -> SeqSchedule {
        let mut schedule = SeqSchedule::default();
        unsafe {
            for s in &GSYSTEMS {
                let s = (s.s)();
                schedule.add_system(s);
            }
        }
        schedule
    }

    pub fn new(gen_terrain: bool) -> Egregoria {
        let mut goria = Egregoria {
            world: Default::default(),
            resources: Default::default(),
        };

        info!("Seed is {}", RNG_SEED);

        unsafe {
            for s in &INIT_FUNCS {
                (s.f)(&mut goria);
            }
        }

        if gen_terrain {
            info!("generating terrain..");
            let t = Instant::now();
            let size = if cfg!(debug_assertions) { 10 } else { 50 };

            goria.map_mut().terrain = Terrain::new(size, size);
            info!("took {}s", t.elapsed().as_secs_f32());

            let c = vec3(3000.0 + 72.2 / 2.0, 200.0 / 2.0 + 1.0, 0.3);
            let obb = OBB::new(c.xy(), -Vec2::X, 72.2, 200.0);

            let [offy, offx] = obb.axis().map(|x| x.normalize().z(0.0));

            let mut tracks = vec![];

            let pat = LanePatternBuilder::new().rail(true).build();

            for i in -1..=1 {
                tracks.push(StraightRoadGen {
                    from: c - offx * (i as f32 * 21.0) - offy * 100.0,
                    to: c - offx * (i as f32 * 21.0) + offy * 120.0,
                    pattern: pat.clone(),
                });
            }

            goria.map_mut().build_special_building(
                &obb,
                BuildingKind::ExternalTrading,
                BuildingGen::NoWalkway {
                    door_pos: Vec2::ZERO,
                },
                &tracks,
            );
        }

        goria
    }

    pub fn world_res(&mut self) -> (&mut World, &mut Resources) {
        (&mut self.world, &mut self.resources)
    }

    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn world_mut_unchecked(&mut self) -> &mut World {
        &mut self.world
    }

    #[profiling::function]
    pub fn tick(&mut self, game_schedule: &mut SeqSchedule, commands: &WorldCommands) -> Duration {
        self.write::<Tick>().0 += 1;
        const WORLD_TICK_DT: f32 = 0.05;

        let t = Instant::now();

        {
            let mut time = self.write::<GameTime>();
            *time = GameTime::new(WORLD_TICK_DT, time.timestamp + WORLD_TICK_DT as f64);
        }

        {
            profiling::scope!("applying commands");
            for command in &commands.commands {
                command.apply(self);
            }
        }

        game_schedule.execute(self);
        t.elapsed()
    }

    pub fn get_tick(&self) -> u32 {
        self.resources.get::<Tick>().unwrap().0
    }

    pub fn hashes(&self) -> BTreeMap<String, u64> {
        let mut hashes = BTreeMap::new();
        let ser = common::saveload::Bincode::encode(&SerWorld(&self.world)).unwrap();
        hashes.insert("world".to_string(), common::hash_u64(&*ser));

        unsafe {
            for l in &SAVELOAD_FUNCS {
                let v = (l.save)(self);
                hashes.insert(l.name.to_string(), common::hash_u64(&*v));
            }
        }

        hashes
    }

    pub fn load_from_disk(save_name: &'static str) -> Option<Self> {
        let goria: Egregoria = common::saveload::CompressedBincode::load(save_name)?;
        Some(goria)
    }

    pub fn save_to_disk(&self, save_name: &'static str) {
        common::saveload::CompressedBincode::save(&self, save_name);
    }

    pub fn pos(&self, e: Entity) -> Option<Vec3> {
        self.comp::<Transform>(e).map(|x| x.position)
    }

    pub(crate) fn add_comp(&mut self, e: Entity, c: impl Component) {
        if self.world.insert_one(e, c).is_err() {
            log::error!("trying to add component to entity but it doesn't exist");
        }
    }

    pub fn comp<T: Component>(&self, e: Entity) -> Option<hecs::Ref<T>> {
        self.world.get::<&T>(e).ok()
    }

    pub fn comp_mut<T: Component>(&mut self, e: Entity) -> Option<hecs::RefMut<T>> {
        self.world.get::<&mut T>(e).ok()
    }

    pub fn write_or_default<T: Resource + Default>(&mut self) -> RefMut<T> {
        self.resources.entry::<T>().or_default()
    }

    pub fn try_write<T: Resource>(&self) -> Option<RefMut<T>> {
        self.resources.get_mut().ok()
    }

    pub fn write<T: Resource>(&self) -> RefMut<T> {
        self.resources
            .get_mut()
            .unwrap_or_else(|_| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn read<T: Resource>(&self) -> Ref<T> {
        self.resources
            .get()
            .unwrap_or_else(|_| panic!("Couldn't fetch resource {}", std::any::type_name::<T>()))
    }

    pub fn map(&self) -> Ref<'_, Map> {
        self.resources.get().unwrap()
    }

    pub(crate) fn map_mut(&self) -> RefMut<'_, Map> {
        self.resources.get_mut().unwrap()
    }

    pub fn insert<T: Resource>(&mut self, res: T) {
        self.resources.insert(res);
    }
}

struct SerWorld<'a>(&'a World);

impl<'a> Serialize for SerWorld<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        hecs::serialize::column::serialize(self.0, &mut SerContext, serializer)
    }
}

impl Serialize for Egregoria {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        log::info!("serializing egregoria");
        let t = Instant::now();
        let mut m: FastMap<String, Vec<u8>> = FastMap::default();

        unsafe {
            for l in &SAVELOAD_FUNCS {
                let v: Vec<u8> = (l.save)(self);
                m.insert(l.name.to_string(), v);
            }
        }

        log::info!("took {}s to serialize resources", t.elapsed().as_secs_f32());

        let v = EgregoriaSer {
            world: SerWorld(&self.world),
            version: VERSION.to_string(),
            res: m,
        }
        .serialize(serializer);
        log::info!("took {}s to serialize in total", t.elapsed().as_secs_f32());
        v
    }
}

#[derive(Serialize)]
struct EgregoriaSer<'a> {
    world: SerWorld<'a>,
    version: String,
    res: FastMap<String, Vec<u8>>,
}

#[derive(Deserialize)]
struct EgregoriaDeser {
    world: DeserWorld,
    version: String,
    res: FastMap<String, Vec<u8>>,
}

impl<'de> Deserialize<'de> for Egregoria {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        log::info!("deserializing egregoria");
        let t = Instant::now();

        let mut goriadeser = EgregoriaDeser::deserialize(deserializer)?;

        log::info!(
            "took {}s to deserialize base deser",
            t.elapsed().as_secs_f32()
        );

        if goriadeser.version != VERSION {
            return Err(Error::custom(format!(
                "couldn't load save, incompatible version! save is: {} - game is: {}",
                goriadeser.version, VERSION
            )));
        }

        let mut goria = Self::new(false);

        goria.world = goriadeser.world.0;

        unsafe {
            for l in &SAVELOAD_FUNCS {
                if let Some(data) = goriadeser.res.remove(l.name) {
                    (l.load)(&mut goria, data);
                }
            }
        }

        log::info!(
            "took {}s to deserialize in total",
            t.elapsed().as_secs_f32()
        );

        Ok(goria)
    }
}

struct DeserWorld(World);

impl<'de> Deserialize<'de> for DeserWorld {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        hecs::serialize::column::deserialize(&mut DeserContext::default(), deserializer)
            .map(DeserWorld)
    }
}

struct SerContext;

#[derive(Default)]
struct DeserContext {
    components: Vec<ComponentId>,
}

macro_rules! register {
    ($($t: ty => $p:ident),+,) => {
        #[derive(Serialize, Deserialize)]
        enum ComponentId {
            $(
                $p,
            )+
        }

        impl hecs::serialize::column::SerializeContext for SerContext {
            fn component_count(&self, archetype: &hecs::Archetype) -> usize {
                archetype.component_types()
                    .filter(|&t| {
                    $(
                        t == std::any::TypeId::of::<$t>() ||
                    )+
                    true
                    })
                    .count()
            }

            fn serialize_component_ids<S: serde::ser::SerializeTuple>(
                &mut self,
                archetype: &hecs::Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                    hecs::serialize::column::try_serialize_id::<$t, _, _>(archetype, &ComponentId::$p, &mut out)?;
                )+
                out.end()
            }

            fn serialize_components<S: serde::ser::SerializeTuple>(
                &mut self,
                archetype: &hecs::Archetype,
                mut out: S,
            ) -> Result<S::Ok, S::Error> {
                $(
                    hecs::serialize::column::try_serialize::<$t, _>(archetype, &mut out)?;
                )+
                out.end()
            }
        }

        impl hecs::serialize::column::DeserializeContext for DeserContext {
            fn deserialize_component_ids<'de, A>(
                &mut self,
                mut seq: A,
            ) -> Result<hecs::ColumnBatchType, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                self.components.clear(); // Discard data from the previous archetype
                let mut batch = hecs::ColumnBatchType::new();
                while let Some(id) = seq.next_element()? {
                    match id {
                        $(
                            ComponentId::$p => {
                                batch.add::<$t>();
                            },
                        )+
                    }
                    self.components.push(id);
                }
                Ok(batch)
            }

            fn deserialize_components<'de, A>(
                &mut self,
                entity_count: u32,
                mut seq: A,
                batch: &mut hecs::ColumnBatchBuilder,
            ) -> Result<(), A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                // Decode component data in the order that the component IDs appeared
                for component in &self.components {
                    match *component {
                        $(
                        ComponentId::$p => {
                            hecs::serialize::column::deserialize_column::<$t, _>(entity_count, &mut seq, batch)?;
                        },
                        )+
                    }
                }
                Ok(())
            }
        }
    };
}

pub struct NoSerialize;

register!(
        Transform => _0,
        Bought => _1,
        BuyFood => _2,
        Collider => _3,
        GoodsCompany => _4,
        Home => _5,
        HumanDecision => _6,
        Itinerary => _7,
        Kinematics => _8,
        Location => _9,
        Pedestrian => _10,
        Router => _11,
        Selectable => _12,
        Sold => _13,
        Vehicle => _14,
        Work => _15,
        Workers => _16,
        Locomotive => _17,
        RailWagon => _18,
        RandomLocomotive => _19,
        ItineraryLeader => _20,
        ItineraryFollower => _21,
        LocomotiveReservation => _22,
        FreightStation => _23,
);
