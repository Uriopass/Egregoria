#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use crate::engine_interaction::WorldCommand;
use crate::map::{BuildingKind, Map};
use crate::map_dynamic::{Itinerary, ItineraryLeader};
use crate::physics::CollisionWorld;
use crate::physics::Speed;
use crate::souls::add_souls_to_empty_buildings;
use crate::souls::goods_company::GoodsCompanyRegistry;
use crate::utils::resources::{Ref, RefMut, Resources};
use common::saveload::Encoder;
use derive_more::{From, TryInto};
use geom::Vec3;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::time::{Duration, Instant};
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
#[cfg(test)]
mod tests;
pub mod transportation;
pub mod utils;
mod world;

pub use world::*;

use crate::engine_interaction::WorldCommand::Init;
use crate::init::{GSYSTEMS, INIT_FUNCS, SAVELOAD_FUNCS};
use crate::utils::scheduler::RunnableSystem;
use crate::utils::time::{Tick, SECONDS_PER_REALTIME_SECOND};
use common::FastMap;
pub use utils::config::*;
pub use utils::par_command_buffer::ParCommandBuffer;
pub use utils::replay::*;

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash, From, TryInto,
)]
pub enum SoulID {
    Human(HumanID),
    GoodsCompany(CompanyID),
    FreightStation(FreightStationID),
}

impl Display for SoulID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SoulID::Human(id) => write!(f, "{:?}", id),
            SoulID::GoodsCompany(id) => write!(f, "{:?}", id),
            SoulID::FreightStation(id) => write!(f, "{:?}", id),
        }
    }
}

impl From<SoulID> for AnyEntity {
    fn from(value: SoulID) -> Self {
        match value {
            SoulID::Human(id) => AnyEntity::HumanID(id),
            SoulID::GoodsCompany(id) => AnyEntity::CompanyID(id),
            SoulID::FreightStation(id) => AnyEntity::FreightStationID(id),
        }
    }
}

impl TryFrom<AnyEntity> for SoulID {
    type Error = ();

    fn try_from(value: AnyEntity) -> Result<Self, Self::Error> {
        match value {
            AnyEntity::HumanID(id) => Ok(SoulID::Human(id)),
            AnyEntity::CompanyID(id) => Ok(SoulID::GoodsCompany(id)),
            AnyEntity::FreightStationID(id) => Ok(SoulID::FreightStation(id)),
            _ => Err(()),
        }
    }
}

debug_inspect_impl!(SoulID);

pub struct Egregoria {
    pub(crate) world: World,
    resources: Resources,
}

const RNG_SEED: u64 = 123;
const VERSION: &str = include_str!("../../VERSION");

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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

    pub fn from_replay(replay: Replay) -> (Egregoria, EgregoriaReplayLoader) {
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

        (
            goria,
            EgregoriaReplayLoader {
                replay,
                pastt: Tick::default(),
                idx: 0,
                speed: 1,
                advance_n_ticks: 0,
            },
        )
    }

    pub fn new_with_options(opts: EgregoriaOptions) -> Egregoria {
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

    pub fn is_equal(&self, other: &Self) -> bool {
        if self.resources.iter().count() != other.resources.iter().count() {
            return false;
        }

        unsafe {
            for l in &SAVELOAD_FUNCS {
                let a = (l.save)(self);
                let b = (l.save)(other);

                if a != b {
                    std::fs::write(format!("{}_a.json", l.name), &*String::from_utf8_lossy(&a))
                        .unwrap();
                    std::fs::write(format!("{}_b.json", l.name), &*String::from_utf8_lossy(&b))
                        .unwrap();
                    return false;
                }
            }
        }

        true
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
            *time = GameTime::new(
                WORLD_TICK_DT,
                time.timestamp + SECONDS_PER_REALTIME_SECOND as f64 * WORLD_TICK_DT as f64,
            );
        }

        game_schedule.execute(self);
        self.write::<Tick>().0 += 1;

        t.elapsed()
    }

    pub fn get_tick(&self) -> u32 {
        self.resources.read::<Tick>().0
    }

    pub fn hashes(&self) -> BTreeMap<String, u64> {
        let mut hashes = BTreeMap::new();
        let ser = common::saveload::Bincode::encode(&self.world).unwrap();
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
        let rep = self.resources.read::<Replay>();
        if rep.enabled {
            common::saveload::JSONPretty::save(&*rep, &format!("{save_name}_replay"));
        }
    }

    pub fn pos<E: WorldTransform>(&self, id: E) -> Option<Vec3> {
        self.world.pos(id)
    }

    pub fn pos_any(&self, id: AnyEntity) -> Option<Vec3> {
        self.world.pos_any(id)
    }

    pub fn get<E: EntityID>(&self, id: E) -> Option<&E::Entity> {
        self.world.get(id)
    }

    pub fn contains(&self, id: AnyEntity) -> bool {
        self.world.contains(id)
    }

    pub fn write_or_default<T: Any + Send + Sync + Default>(&mut self) -> RefMut<T> {
        self.resources.write_or_default::<T>()
    }

    pub fn try_write<T: Any + Send + Sync>(&self) -> Option<RefMut<T>> {
        self.resources.try_write().ok()
    }

    pub fn write<T: Any + Send + Sync>(&self) -> RefMut<T> {
        self.resources.write()
    }

    pub fn read<T: Any + Send + Sync>(&self) -> Ref<T> {
        self.resources.read()
    }

    pub fn map(&self) -> Ref<'_, Map> {
        self.resources.read()
    }

    pub(crate) fn map_mut(&self) -> RefMut<'_, Map> {
        self.resources.write()
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, res: T) {
        self.resources.insert(res);
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
            world: &self.world,
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
    world: &'a World,
    version: String,
    res: FastMap<String, Vec<u8>>,
}

#[derive(Deserialize)]
struct EgregoriaDeser {
    world: World,
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

        let cur_version_parts = VERSION.split('.').collect::<Vec<_>>();
        let deser_parts = goriadeser.version.split('.').collect::<Vec<_>>();

        if cur_version_parts[0] != deser_parts[0]
            || (cur_version_parts[0] == "0" && cur_version_parts[1] != deser_parts[1])
        {
            log::warn!(
                "incompatible version, save might be corrupted! save is: {} - game is: {}",
                goriadeser.version,
                VERSION
            );
        }

        let mut goria = Self {
            world: World::default(),
            resources: Resources::default(),
        };

        unsafe {
            for s in &INIT_FUNCS {
                (s.f)(&mut goria);
            }
        }

        goria.world = goriadeser.world;

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
      0,
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
      0,
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
      0,
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
      0,
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
      0,
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
      0,
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
      0,
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
      0,
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
      0,
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
    [ 0,
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
