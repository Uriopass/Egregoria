use crate::economy::Government;
use crate::map::procgen::{load_parismap, load_testfield};
use crate::map::{
    BuildingGen, BuildingID, BuildingKind, IntersectionID, LaneID, LanePattern, LanePatternBuilder,
    LightPolicy, LotID, Map, MapProject, RoadID, StraightRoadGen, Terrain, TurnPolicy,
};
use crate::map_dynamic::BuildingInfos;
use crate::utils::time::{GameTime, Tick};
use crate::vehicles::trains::{spawn_train, RailWagonKind};
use crate::{Egregoria, EgregoriaOptions, Replay};
use geom::{vec3, Transform, Vec2, OBB};
use hecs::Entity;
use serde::{Deserialize, Serialize};
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
    GenerateTerrain(u32),
    MapRemoveIntersection(IntersectionID),
    MapRemoveRoad(RoadID),
    MapRemoveBuilding(BuildingID),
    MapBuildHouse(LotID),
    AddTrain(f32, u32, LaneID),
    MapMakeConnection(MapProject, MapProject, Option<Vec2>, LanePattern), // todo: allow lane pattern builder
    MapUpdateIntersectionPolicy(IntersectionID, TurnPolicy, LightPolicy),
    MapBuildSpecialBuilding(OBB, BuildingKind, BuildingGen, Vec<StraightRoadGen>),
    MapLoadParis,
    MapLoadTestField(Vec2, u32, f32),
    ResetSave,
    SetGameTime(GameTime),
    UpdateTransform(Entity, Transform),
}

impl WorldCommands {
    pub fn push(&mut self, cmd: WorldCommand) {
        self.commands.push(cmd);
    }

    pub fn merge(&mut self, src: &WorldCommands) {
        self.commands.extend_from_slice(&src.commands);
    }

    pub fn iter(&self) -> impl Iterator<Item = &WorldCommand> {
        self.commands.iter()
    }

    pub fn as_ref(&self) -> &[WorldCommand] {
        &self.commands
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn map_load_paris(&mut self) {
        self.commands.push(MapLoadParis)
    }

    pub fn map_load_testfield(&mut self, pos: Vec2, size: u32, spacing: f32) {
        self.commands.push(MapLoadTestField(pos, size, spacing))
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
        self.commands.push(AddTrain(dist, n_wagons, laneid))
    }

    pub fn map_build_special_building(
        &mut self,
        obb: OBB,
        kind: BuildingKind,
        gen: BuildingGen,
        attachments: Vec<StraightRoadGen>,
    ) {
        self.commands
            .push(MapBuildSpecialBuilding(obb, kind, gen, attachments))
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
        self.commands
            .push(MapMakeConnection(from, to, interpoint, pat))
    }

    pub fn map_update_intersection_policy(
        &mut self,
        id: IntersectionID,
        tp: TurnPolicy,
        lp: LightPolicy,
    ) {
        self.commands.push(MapUpdateIntersectionPolicy(id, tp, lp))
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
            MapMakeConnection(from, to, interpoint, ref pat) => {
                goria
                    .write::<Map>()
                    .make_connection(from, to, interpoint, pat);
            }
            MapUpdateIntersectionPolicy(id, tp, lp) => {
                goria.map_mut().update_intersection(id, move |i| {
                    i.light_policy = lp;
                    i.turn_policy = tp;
                })
            }
            MapBuildSpecialBuilding(obb, kind, gen, ref attachments) => {
                if let Some(id) =
                    goria
                        .write::<Map>()
                        .build_special_building(&obb, kind, gen, attachments)
                {
                    goria.write::<BuildingInfos>().insert(id);
                }
            }
            SetGameTime(gt) => *goria.write::<GameTime>() = gt,
            AddTrain(dist, n_wagons, lane) => {
                spawn_train(goria, dist, n_wagons, lane, RailWagonKind::Fret);
            }
            MapLoadParis => load_parismap(&mut *goria.map_mut()),
            MapLoadTestField(pos, size, spacing) => {
                load_testfield(&mut *goria.map_mut(), pos, size, spacing)
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
            GenerateTerrain(size) => {
                generate_terrain(goria, size);
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

    if goria
        .map_mut()
        .build_special_building(
            &obb,
            BuildingKind::ExternalTrading,
            BuildingGen::NoWalkway {
                door_pos: Vec2::ZERO,
            },
            &tracks,
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
