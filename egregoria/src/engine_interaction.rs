use crate::economy::Government;
use crate::map::procgen::{load_parismap, load_testfield};
use crate::map::{
    BuildingGen, BuildingID, BuildingKind, IntersectionID, LaneID, LanePattern, LanePatternBuilder,
    LightPolicy, LotID, Map, MapProject, ProjectKind, RoadID, Terrain, TurnPolicy,
};
use crate::map_dynamic::BuildingInfos;
use crate::transportation::train::{spawn_train, RailWagonKind};
use crate::utils::time::{GameTime, Tick};
use crate::{Egregoria, EgregoriaOptions, Replay};
use geom::{vec3, Polygon, Transform, Vec2, OBB};
use hecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;
use WorldCommand::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Selectable {
    pub radius: f32,
}

impl Selectable {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Default for Selectable {
    fn default() -> Self {
        Self { radius: 5.0 }
    }
}

#[derive(Clone, Default)]
pub struct WorldCommands {
    pub(crate) commands: Vec<WorldCommand>,
}

defer_serialize!(WorldCommands, Vec<WorldCommand>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldCommand {
    Init(Box<EgregoriaOptions>),
    MapRemoveIntersection(IntersectionID),
    MapRemoveRoad(RoadID),
    MapRemoveBuilding(BuildingID),
    MapBuildHouse(LotID),
    AddTrain {
        dist: f32,
        n_wagons: u32,
        lane: LaneID,
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
        zone: Option<Polygon>,
    },
    MapLoadParis,
    MapLoadTestField {
        pos: Vec2,
        size: u32,
        spacing: f32,
    },
    UpdateZone {
        building: BuildingID,
        zone: Polygon,
    },
    ResetSave,
    SetGameTime(GameTime),
    UpdateTransform(Entity, Transform),
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

    pub fn update_transform(&mut self, e: Entity, trans: Transform) {
        self.commands.push(UpdateTransform(e, trans))
    }

    pub fn reset_save(&mut self) {
        self.commands.push(ResetSave)
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
        zone: Option<Polygon>,
    ) {
        self.commands.push(MapBuildSpecialBuilding {
            pos: obb,
            kind,
            gen,
            zone,
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
    pub(crate) fn apply(&self, goria: &mut Egregoria) {
        let cost = Government::action_cost(self, goria);
        goria.write::<Government>().money -= cost;

        let mut rep = goria.resources.get_mut::<Replay>().unwrap();
        if rep.enabled {
            let tick = goria.read::<Tick>();
            rep.commands.push((*tick, self.clone()));
        }
        drop(rep);

        match *self {
            MapRemoveIntersection(id) => goria.map_mut().remove_intersection(id),
            MapRemoveRoad(id) => drop(goria.map_mut().remove_road(id)),
            MapRemoveBuilding(id) => drop(goria.map_mut().remove_building(id)),
            MapBuildHouse(id) => {
                if let Some(build) = goria.map_mut().build_house(id) {
                    let mut infos = goria.write::<BuildingInfos>();
                    infos.insert(build);
                }
            }
            MapMakeConnection {
                from,
                to,
                inter,
                ref pat,
            } => {
                goria.write::<Map>().make_connection(from, to, inter, pat);
            }
            MapMakeMultipleConnections(ref projects, ref links) => {
                let mut map = goria.map_mut();
                let mut inters = BTreeMap::new();
                for (from, to, interpoint, pat) in links {
                    let mut fromproj = projects[*from];
                    let mut toproj = projects[*to];

                    if let Some(i) = inters.get(from) {
                        fromproj.kind = ProjectKind::Inter(*i);
                    }
                    if let Some(i) = inters.get(to) {
                        toproj.kind = ProjectKind::Inter(*i);
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
            } => goria.map_mut().update_intersection(id, move |i| {
                i.light_policy = lp;
                i.turn_policy = tp;
            }),
            MapBuildSpecialBuilding {
                pos: obb,
                kind,
                gen,
                ref zone,
            } => {
                if let Some(id) =
                    goria
                        .write::<Map>()
                        .build_special_building(&obb, kind, gen, zone.clone())
                {
                    goria.write::<BuildingInfos>().insert(id);
                }
            }
            SetGameTime(gt) => *goria.write::<GameTime>() = gt,
            AddTrain {
                dist,
                n_wagons,
                lane,
            } => {
                spawn_train(goria, dist, n_wagons, lane, RailWagonKind::Fret);
            }
            MapLoadParis => load_parismap(&mut goria.map_mut()),
            MapLoadTestField { pos, size, spacing } => {
                load_testfield(&mut goria.map_mut(), pos, size, spacing)
            }
            ResetSave => {
                let opts = goria.read::<EgregoriaOptions>().clone();
                *goria = Egregoria::new_with_options(opts);
            }
            UpdateTransform(e, t) => {
                if let Some(mut x) = goria.comp_mut(e) {
                    *x = t
                }
            }
            Init(ref opts) => {
                if opts.save_replay {
                    let mut rep = goria.resources.get_mut::<Replay>().unwrap();
                    rep.enabled = true;
                    let tick = goria.read::<Tick>();
                    rep.commands.push((*tick, Init(opts.clone())));
                }

                if opts.terrain_size > 0 {
                    generate_terrain(goria, opts.terrain_size);
                }

                goria
                    .resources
                    .insert::<EgregoriaOptions>(EgregoriaOptions::clone(opts));
            }
            UpdateZone { building, ref zone } => {
                let mut map = goria.map_mut();

                map.update_zone(building, move |z| *z = zone.clone());
            }
        }
    }
}

fn generate_terrain(goria: &mut Egregoria, size: u32) {
    info!("generating terrain..");
    let t = Instant::now();

    goria.map_mut().terrain = Terrain::new(size, size);
    info!("took {}s", t.elapsed().as_secs_f32());

    let c = vec3(3000.0 + 72.2 / 2.0, 200.0 / 2.0 + 1.0, 0.3);
    let obb = OBB::new(c.xy(), -Vec2::X, 72.2, 200.0);

    let [offy, _] = obb.axis().map(|x| x.normalize().z(0.0));

    let pat = LanePatternBuilder::new().rail(true).build();

    goria.map_mut().make_connection(
        MapProject::ground(c - offy * 100.0),
        MapProject::ground(c + offy * 120.0),
        None,
        &pat,
    );

    if goria
        .map_mut()
        .build_special_building(
            &obb,
            BuildingKind::ExternalTrading,
            BuildingGen::NoWalkway {
                door_pos: Vec2::ZERO,
            },
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
            commands: iter.into_iter().flat_map(|x| x.commands).collect(),
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
