use std::collections::BTreeMap;
use std::time::Instant;

use prototypes::RollingStockID;
use serde::{Deserialize, Serialize};

use geom::{vec3, Vec2, Vec3, OBB};
use prototypes::BuildingGen;
use prototypes::GameTime;
use WorldCommand::*;

use crate::economy::Government;
use crate::map::procgen::{load_parismap, load_testfield};
use crate::map::{
    BuildingID, BuildingKind, Environment, IntersectionID, LaneID, LanePattern, LanePatternBuilder,
    LightPolicy, LotID, Map, MapProject, ProjectKind, RoadID, TerraformKind, TurnPolicy, Zone,
};
use crate::map_dynamic::{BuildingInfos, ParkingManagement};
use crate::multiplayer::chat::Message;
use crate::multiplayer::MultiplayerState;
use crate::transportation::testing_vehicles::RandomVehicles;
use crate::transportation::train::{spawn_train, RailWagonKind};
use crate::transportation::{spawn_parked_vehicle_with_spot, unpark, VehicleKind};
use crate::utils::rand_provider::RandProvider;
use crate::{Replay, Simulation, SimulationOptions};

#[derive(Clone, Default)]
pub struct WorldCommands {
    pub(crate) commands: Vec<WorldCommand>,
}

defer_serialize!(WorldCommands, Vec<WorldCommand>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldCommand {
    Init(Box<SimulationOptions>),
    MapRemoveIntersection(IntersectionID),
    MapRemoveRoad(RoadID),
    MapRemoveBuilding(BuildingID),
    MapBuildHouse(LotID),
    Terraform {
        kind: TerraformKind,
        center: Vec2,
        radius: f32,
        amount: f32,
        level: f32,                  // only for flatten
        slope: Option<(Vec3, Vec3)>, // start and end of slope
    },
    SendMessage {
        message: Message,
    },
    SpawnRandomCars {
        n_cars: usize,
    },
    AddTrain {
        dist: f32,
        n_wagons: u32,
        lane: LaneID,
    },
    SpawnTrain {
        wagons: Vec<RollingStockID>,
        lane: LaneID,
        dist: f32,
    },
    MapMakeConnection {
        from: MapProject,
        to: MapProject,
        inter: Option<Vec2>,
        pat: LanePattern,
    }, // todo: allow lane pattern builder
    MapMakeMultipleConnections(
        Vec<MapProject>,
        Vec<(usize, usize, Option<Vec2>, LanePattern)>,
    ),
    MapUpdateIntersectionPolicy {
        inter: IntersectionID,
        turn: TurnPolicy,
        light: LightPolicy,
    },
    MapBuildSpecialBuilding {
        pos: OBB,
        kind: BuildingKind,
        gen: BuildingGen,
        #[serde(default)]
        zone: Option<Zone>,
        #[serde(default)]
        connected_road: Option<RoadID>,
    },
    MapLoadParis,
    MapLoadTestField {
        pos: Vec2,
        size: u32,
        spacing: f32,
    },
    UpdateZone {
        building: BuildingID,
        zone: Zone,
    },
    SetGameTime(GameTime),
}

impl AsRef<[WorldCommand]> for WorldCommands {
    fn as_ref(&self) -> &[WorldCommand] {
        &self.commands
    }
}

impl WorldCommands {
    pub fn push(&mut self, cmd: WorldCommand) {
        self.commands.push(cmd);
    }

    pub fn extend(&mut self, cmds: impl IntoIterator<Item = WorldCommand>) {
        self.commands.extend(cmds);
    }

    pub fn merge(&mut self, src: &WorldCommands) {
        self.commands.extend_from_slice(&src.commands);
    }

    pub fn iter(&self) -> impl Iterator<Item = &WorldCommand> {
        self.commands.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn map_load_paris(&mut self) {
        self.commands.push(MapLoadParis)
    }

    pub fn map_load_testfield(&mut self, pos: Vec2, size: u32, spacing: f32) {
        self.commands.push(MapLoadTestField { pos, size, spacing })
    }

    pub fn set_game_time(&mut self, gt: GameTime) {
        self.commands.push(SetGameTime(gt))
    }

    pub fn add_train(&mut self, dist: f32, n_wagons: u32, laneid: LaneID) {
        self.commands.push(AddTrain {
            dist,
            n_wagons,
            lane: laneid,
        })
    }

    pub fn map_build_special_building(
        &mut self,
        obb: OBB,
        kind: BuildingKind,
        gen: BuildingGen,
        zone: Option<Zone>,
        connected_road: Option<RoadID>,
    ) {
        self.commands.push(MapBuildSpecialBuilding {
            pos: obb,
            kind,
            gen,
            zone,
            connected_road,
        })
    }

    pub fn map_remove_intersection(&mut self, id: IntersectionID) {
        self.commands.push(MapRemoveIntersection(id))
    }

    pub fn map_remove_road(&mut self, id: RoadID) {
        self.commands.push(MapRemoveRoad(id))
    }

    pub fn map_remove_building(&mut self, id: BuildingID) {
        self.commands.push(MapRemoveBuilding(id))
    }

    pub fn map_build_house(&mut self, id: LotID) {
        self.commands.push(MapBuildHouse(id))
    }

    pub fn map_make_connection(
        &mut self,
        from: MapProject,
        to: MapProject,
        interpoint: Option<Vec2>,
        pat: LanePattern,
    ) {
        self.commands.push(MapMakeConnection {
            from,
            to,
            inter: interpoint,
            pat,
        })
    }

    pub fn map_update_intersection_policy(
        &mut self,
        id: IntersectionID,
        tp: TurnPolicy,
        lp: LightPolicy,
    ) {
        self.commands.push(MapUpdateIntersectionPolicy {
            inter: id,
            turn: tp,
            light: lp,
        })
    }
}

impl WorldCommand {
    /// Returns true if the command can be applied without any systems needed to be run afterward
    pub fn is_instant(&self) -> bool {
        matches!(
            self,
            MapBuildHouse(_)
                | MapUpdateIntersectionPolicy { .. }
                | UpdateZone { .. }
                | SetGameTime(_)
        )
    }

    pub fn apply(&self, sim: &mut Simulation) {
        let cost = Government::action_cost(self, sim);
        sim.write::<Government>().money -= cost;

        let mut rep = sim.resources.write::<Replay>();
        if rep.enabled {
            let tick = sim.read::<GameTime>().tick;
            rep.push(tick, self.clone());
        }
        drop(rep);

        match *self {
            MapRemoveIntersection(id) => sim.map_mut().remove_intersection(id),
            MapRemoveRoad(id) => drop(sim.map_mut().remove_road(id)),
            MapRemoveBuilding(id) => drop(sim.map_mut().remove_building(id)),
            MapBuildHouse(id) => {
                if let Some(build) = sim.map_mut().build_house(id) {
                    let mut infos = sim.write::<BuildingInfos>();
                    infos.insert(build);
                }
            }
            MapMakeConnection {
                from,
                to,
                inter,
                ref pat,
            } => {
                sim.write::<Map>().make_connection(from, to, inter, pat);
            }
            MapMakeMultipleConnections(ref projects, ref links) => {
                let mut map = sim.map_mut();
                let mut inters = BTreeMap::new();
                for (from, to, interpoint, pat) in links {
                    let mut fromproj = projects[*from];
                    let mut toproj = projects[*to];

                    if let Some(i) = inters.get(from) {
                        fromproj.kind = ProjectKind::Intersection(*i);
                    }
                    if let Some(i) = inters.get(to) {
                        toproj.kind = ProjectKind::Intersection(*i);
                    }

                    if let Some((_, r)) = map.make_connection(fromproj, toproj, *interpoint, pat) {
                        if fromproj.kind.is_ground() {
                            inters.insert(*from, map.roads[r].src);
                        }
                        if toproj.kind.is_ground() {
                            inters.insert(*to, map.roads[r].dst);
                        }
                    }
                }
            }
            MapUpdateIntersectionPolicy {
                inter: id,
                turn: tp,
                light: lp,
            } => sim.map_mut().update_intersection(id, move |i| {
                i.light_policy = lp;
                i.turn_policy = tp;
            }),
            MapBuildSpecialBuilding {
                pos: obb,
                kind,
                gen,
                ref zone,
                connected_road,
            } => {
                if let Some(id) = sim.write::<Map>().build_special_building(
                    &obb,
                    kind,
                    gen,
                    zone.clone(),
                    connected_road,
                ) {
                    sim.write::<BuildingInfos>().insert(id);
                }
            }
            SetGameTime(gt) => *sim.write::<GameTime>() = gt,
            AddTrain {
                dist: _,
                n_wagons: _,
                lane: _,
            } => {}
            SpawnTrain {
                ref wagons,
                lane,
                dist,
            } => {
                spawn_train(sim, wagons, RailWagonKind::Freight, lane, dist);
            }

            MapLoadParis => load_parismap(&mut sim.map_mut()),
            MapLoadTestField { pos, size, spacing } => {
                load_testfield(&mut sim.map_mut(), pos, size, spacing)
            }
            Init(ref opts) => {
                if opts.save_replay {
                    let mut rep = sim.resources.write::<Replay>();
                    rep.enabled = true;
                    let tick = sim.read::<GameTime>().tick;
                    rep.push(tick, Init(opts.clone()));
                }

                if opts.terrain_size > 0 {
                    generate_terrain(sim, opts.terrain_size);
                }

                sim.resources
                    .insert::<SimulationOptions>(SimulationOptions::clone(opts));
            }
            UpdateZone { building, ref zone } => {
                let mut map = sim.map_mut();

                map.update_zone(building, move |z| *z = zone.clone());
            }
            SpawnRandomCars { n_cars } => {
                for _ in 0..n_cars {
                    let mut pm = sim.write::<ParkingManagement>();
                    let map = sim.map();
                    let mut rng = sim.write::<RandProvider>();

                    let Some(spot) = pm.reserve_random_free_spot(&map.parking, rng.next_u64())
                    else {
                        continue;
                    };

                    drop((map, pm, rng));

                    let Some(v_id) = spawn_parked_vehicle_with_spot(sim, VehicleKind::Car, spot)
                    else {
                        continue;
                    };
                    unpark(sim, v_id);

                    sim.write::<RandomVehicles>().vehicles.insert(v_id);
                }
            }
            SendMessage { ref message } => {
                sim.write::<MultiplayerState>()
                    .chat
                    .add_message(message.clone());
            }
            Terraform {
                kind,
                amount,
                center,
                radius,
                level,
                slope,
            } => {
                let tick = sim.read::<GameTime>().tick;
                sim.map_mut()
                    .terraform(tick, kind, center, radius, amount, level, slope);
            }
        }
    }
}

fn generate_terrain(sim: &mut Simulation, size: u16) {
    info!("generating terrain..");
    let t = Instant::now();

    sim.map_mut().environment = Environment::new(size, size);
    info!("took {}s", t.elapsed().as_secs_f32());

    let c = vec3(3000.0 + 72.2 / 2.0, 200.0 / 2.0 + 1.0, 0.0);
    let obb = OBB::new(c.xy(), -Vec2::X, 72.2, 200.0);

    let [offy, _] = obb.axis().map(|x| x.normalize().z(0.0));

    let pat = LanePatternBuilder::new().rail(true).build();

    sim.map_mut().make_connection(
        MapProject::ground(c - offy * 100.0),
        MapProject::ground(c + offy * 120.0),
        None,
        &pat,
    );

    if sim
        .map_mut()
        .build_special_building(
            &obb,
            BuildingKind::ExternalTrading,
            BuildingGen::NoWalkway {
                door_pos: Vec2::ZERO,
            },
            None,
            None,
        )
        .is_none()
    {
        log::error!("failed to build external trading");
    }
}

impl FromIterator<WorldCommands> for WorldCommands {
    fn from_iter<T: IntoIterator<Item = WorldCommands>>(iter: T) -> Self {
        Self {
            commands: iter
                .into_iter()
                .flat_map(|x: WorldCommands| x.commands)
                .collect(),
        }
    }
}

impl From<Vec<WorldCommand>> for WorldCommands {
    fn from(commands: Vec<WorldCommand>) -> Self {
        Self { commands }
    }
}

impl From<&WorldCommands> for Vec<WorldCommand> {
    fn from(x: &WorldCommands) -> Self {
        x.commands.clone()
    }
}
