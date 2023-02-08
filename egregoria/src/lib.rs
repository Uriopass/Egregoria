#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use crate::economy::{Bought, Sold, Workers};
use crate::engine_interaction::{Selectable, WorldCommand};
use crate::map::{BuildingKind, Map};
use crate::map_dynamic::{DispatchKind, Itinerary, ItineraryFollower, ItineraryLeader, Router};
use crate::physics::CollisionWorld;
use crate::physics::{Collider, Speed};
use crate::souls::add_souls_to_empty_buildings;
use crate::souls::desire::{BuyFood, Home, Work};
use crate::souls::goods_company::{GoodsCompany, GoodsCompanyRegistry};
use crate::souls::human::HumanDecision;
use crate::transportation::train::{Locomotive, LocomotiveReservation};
use crate::transportation::{Pedestrian, Vehicle};
use common::saveload::Encoder;
use geom::{Transform, Vec3};
use hecs::{Component, Entity, World};
use resources::{Ref, RefMut, Resource, Resources};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::time::{Duration, Instant};
use transportation::Location;
use utils::rand_provider::RandProvider;
use utils::scheduler::SeqSchedule;
use utils::time::{GameTime, SECONDS_PER_DAY, SECONDS_PER_HOUR};

#[macro_use]
extern crate common;

#[allow(unused_imports)]
#[macro_use]
extern crate inline_tweak;

#[macro_use]
extern crate egui_inspect;

#[macro_use]
extern crate log as extern_log;

pub mod economy;
pub mod engine_interaction;
pub mod init;
pub mod map;
pub mod map_dynamic;
pub mod physics;
pub mod souls;
mod tests;
pub mod transportation;
pub mod utils;

use crate::engine_interaction::WorldCommand::Init;
use crate::init::{GSYSTEMS, INIT_FUNCS, SAVELOAD_FUNCS};
use crate::souls::freight_station::FreightStation;
use crate::transportation::train::RailWagon;
use crate::utils::scheduler::RunnableSystem;
use crate::utils::time::Tick;
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

const RNG_SEED: u64 = 123;
const VERSION: &str = include_str!("../../VERSION");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgregoriaOptions {
    pub terrain_size: u32,
    pub save_replay: bool,
}

impl Default for EgregoriaOptions {
    fn default() -> Self {
        EgregoriaOptions {
            terrain_size: 50,
            save_replay: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Replay {
    pub enabled: bool,
    pub commands: Vec<(Tick, WorldCommand)>,
}

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
        Self::new_with_options(EgregoriaOptions {
            terrain_size: if gen_terrain { 50 } else { 0 },
            ..Default::default()
        })
    }

    pub fn from_replay(replay: Replay) -> Egregoria {
        Self::_new(EgregoriaOptions::default(), Some(replay))
    }

    pub fn new_with_options(opts: EgregoriaOptions) -> Egregoria {
        Self::_new(opts, None)
    }

    fn _new(opts: EgregoriaOptions, replay: Option<Replay>) -> Egregoria {
        let mut goria = Egregoria {
            world: Default::default(),
            resources: Default::default(),
        };

        info!("Seed is {}", RNG_SEED);
        info!("{:?}", opts);

        unsafe {
            for s in &INIT_FUNCS {
                (s.f)(&mut goria);
            }
        }

        if let Some(replay) = replay {
            let mut schedule = Egregoria::schedule();
            let mut pastt = Tick::default();
            let mut idx = 0;

            // iterate through tick grouped commands
            while idx < replay.commands.len() {
                let curt = replay.commands[idx].0;
                while pastt < curt {
                    goria.tick(&mut schedule, &[]);
                    pastt.0 += 1;
                }

                let idx_start = idx;
                while idx < replay.commands.len() && replay.commands[idx].0 == curt {
                    idx += 1;
                }
                let command_slice = &replay.commands[idx_start..idx];

                log::info!("[replay] acttick {:?} ({})", pastt, command_slice.len());
                goria.tick(&mut schedule, command_slice.iter().map(|(_, c)| c));
                pastt.0 += 1;
            }

            return goria;
        }

        Init(Box::new(opts)).apply(&mut goria);

        let start_commands: Vec<(u32, WorldCommand)> =
            common::saveload::JSON::decode(START_COMMANDS.as_bytes()).unwrap();

        for (_, command) in start_commands {
            command.apply(&mut goria);
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
    pub fn tick<'a>(
        &mut self,
        game_schedule: &mut SeqSchedule,
        commands: impl IntoIterator<Item = &'a WorldCommand>,
    ) -> Duration {
        let t = Instant::now();
        // It is very important that the first thing being done is applying commands
        // so that instant commands work on single player but the game is still deterministic
        {
            profiling::scope!("applying commands");
            for command in commands {
                command.apply(self);
            }
        }

        const WORLD_TICK_DT: f32 = 0.05;
        {
            let mut time = self.write::<GameTime>();
            *time = GameTime::new(WORLD_TICK_DT, time.timestamp + WORLD_TICK_DT as f64);
        }

        game_schedule.execute(self);
        self.write::<Tick>().0 += 1;

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

    pub fn load_replay_from_disk(save_name: &str) -> Option<Replay> {
        let path = format!("{save_name}_replay");
        let replay: Replay = common::saveload::JSON::load(&path)?;
        Some(replay)
    }

    pub fn load_from_disk(save_name: &str) -> Option<Self> {
        let goria: Egregoria = common::saveload::CompressedBincode::load(save_name)?;
        Some(goria)
    }

    pub fn save_to_disk(&self, save_name: &str) {
        common::saveload::CompressedBincode::save(&self, save_name);
        let rep = self.resources.get::<Replay>().unwrap();
        if rep.enabled {
            common::saveload::JSONPretty::save(&*rep, &format!("{save_name}_replay"));
        }
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

        let mut goriadeser = <EgregoriaDeser as Deserialize>::deserialize(deserializer)?;

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
                    false
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
        Speed => _8,
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
        ItineraryLeader => _20,
        ItineraryFollower => _21,
        LocomotiveReservation => _22,
        FreightStation => _23,
        DispatchKind => _24,
);

const START_COMMANDS: &str = r#"
[
     [
      0,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              6514.85,
              9394.27,
              0.31
            ],
            "kind": "Ground"
          },
          "to": {
            "pos": [
              6333.2446,
              9478.051,
              0.31
            ],
            "kind": "Ground"
          },
          "inter": null,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": []
          }
        }
      }
    ],
    [
      0,
      {
        "MapBuildSpecialBuilding": {
          "pos": {
            "corners": [
              5028940425962066516,
              5029683901981058912,
              5028408391183496666,
              5027664915164504270
            ]
          },
          "kind": "RailFreightStation",
          "gen": {
            "NoWalkway": {
              "door_pos": 0
            }
          }
        }
      }
    ],
    [
      1,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              3036.1,
              221.0,
              0.3
            ],
            "kind": {
              "Inter": {
                "idx": 2,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              4246.6924,
              3259.3774,
              0.31
            ],
            "kind": "Ground"
          },
          "inter": 4991505619838112483,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": [
              [
                "Rail",
                9.0
              ]
            ]
          }
        }
      }
    ],
    [
      2,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              4246.6924,
              3259.3774,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 5,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              6147.913,
              7315.1196,
              0.31
            ],
            "kind": "Ground"
          },
          "inter": 5024504635272202713,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": [
              [
                "Rail",
                9.0
              ]
            ]
          }
        }
      }
    ],
    [
      3,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              6147.913,
              7315.1196,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 6,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              6271.2505,
              8647.945,
              0.31
            ],
            "kind": "Ground"
          },
          "inter": 5026585735910612163,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": [
              [
                "Rail",
                9.0
              ]
            ]
          }
        }
      }
    ],
    [
      4,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              6271.2505,
              8647.945,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 7,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              6469.385,
              8973.193,
              0.31
            ],
            "kind": "Ground"
          },
          "inter": 5027576078060451273,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": []
          }
        }
      }
    ],
    [
      5,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              6469.385,
              8973.193,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 8,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              6627.6006,
              9225.364,
              0.31
            ],
            "kind": "Ground"
          },
          "inter": 5030126116108291542,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": []
          }
        }
      }
    ],
    [
      6,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              6627.6006,
              9225.364,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 9,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              6514.85,
              9394.27,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 3,
                "version": 1
              }
            }
          },
          "inter": 5030305808950368141,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": []
          }
        }
      }
    ],
    [
      7,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              6333.2446,
              9478.051,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 4,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              6120.8,
              9363.014,
              0.31
            ],
            "kind": "Ground"
          },
          "inter": 5026096268554115102,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": []
          }
        }
      }
    ],
    [
      8,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              6120.8,
              9363.014,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 10,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              6202.4365,
              8983.185,
              0.31
            ],
            "kind": "Ground"
          },
          "inter": 5025518762956125798,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": []
          }
        }
      }
    ],
    [
      9,
      {
        "MapMakeConnection": {
          "from": {
            "pos": [
              6202.4365,
              8983.185,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 11,
                "version": 1
              }
            }
          },
          "to": {
            "pos": [
              6271.2505,
              8647.945,
              0.31
            ],
            "kind": {
              "Inter": {
                "idx": 7,
                "version": 1
              }
            }
          },
          "inter": 5026925811421779699,
          "pat": {
            "lanes_forward": [
              [
                "Rail",
                9.0
              ]
            ],
            "lanes_backward": []
          }
        }
      }
    ],
    [ 10,
      {
        "AddTrain": {
          "dist": 150.0,
          "n_wagons": 7,
          "lane": {
            "idx": 3,
            "version": 1
          }
        }
      }
    ]
]
"#;
