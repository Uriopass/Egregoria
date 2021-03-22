use crate::Egregoria;
use map_model::{
    BuildingGen, BuildingID, BuildingKind, IntersectionID, LanePattern, LightPolicy, LotID,
    LotKind, Map, MapProject, RoadID, TurnPolicy,
};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct WorldCommands {
    pub(crate) commands: Vec<WorldCommand>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum WorldCommand {
    MapRemoveIntersection(IntersectionID),
    MapRemoveRoad(RoadID),
    MapRemoveBuilding(BuildingID),
    MapSetLotKind(LotID, LotKind),
    MapMakeConnection(MapProject, MapProject, Option<Vec2>, LanePattern),
    MapUpdateIntersectionPolicy(IntersectionID, TurnPolicy, LightPolicy),
    MapBuildSpecialBuilding(RoadID, OBB, BuildingKind, BuildingGen),
    MapBuildHouses,
    MapLoadParis,
    MapLoadTestField,
    MapClear,
    SetGameTime(GameTime),
    MapGenerateTrees(AABB),
}

use crate::map_dynamic::BuildingInfos;
use common::GameTime;
use geom::{Vec2, AABB, OBB};
use WorldCommand::*;

impl WorldCommands {
    pub fn merge(&mut self, src: impl Iterator<Item = WorldCommand>) {
        self.commands.extend(src);
    }

    pub fn iter(&self) -> impl Iterator<Item = &WorldCommand> {
        self.commands.iter()
    }

    pub fn map_generate_trees(&mut self, aabb: AABB) {
        self.commands.push(MapGenerateTrees(aabb));
    }

    pub fn map_load_paris(&mut self) {
        self.commands.push(MapLoadParis)
    }

    pub fn map_load_testfield(&mut self) {
        self.commands.push(MapLoadTestField)
    }

    pub fn map_clear(&mut self) {
        self.commands.push(MapClear)
    }

    pub fn map_build_houses(&mut self) {
        self.commands.push(MapBuildHouses);
    }

    pub fn set_game_time(&mut self, gt: GameTime) {
        self.commands.push(SetGameTime(gt))
    }

    pub fn map_build_special_building(
        &mut self,
        id: RoadID,
        obb: OBB,
        kind: BuildingKind,
        gen: BuildingGen,
    ) {
        self.commands
            .push(MapBuildSpecialBuilding(id, obb, kind, gen))
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

    pub fn map_set_lot_kind(&mut self, id: LotID, kind: LotKind) {
        self.commands.push(MapSetLotKind(id, kind))
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
        match *self {
            MapRemoveIntersection(id) => goria.write::<Map>().remove_intersection(id),
            MapRemoveRoad(id) => drop(goria.write::<Map>().remove_road(id)),
            MapRemoveBuilding(id) => drop(goria.write::<Map>().remove_building(id)),
            MapSetLotKind(id, kind) => goria.write::<Map>().set_lot_kind(id, kind),
            MapMakeConnection(from, to, interpoint, ref pat) => {
                goria
                    .write::<Map>()
                    .make_connection(from, to, interpoint, pat);
            }
            MapUpdateIntersectionPolicy(id, tp, lp) => {
                goria.write::<Map>().update_intersection(id, move |i| {
                    i.light_policy = lp;
                    i.turn_policy = tp;
                })
            }
            MapBuildSpecialBuilding(id, obb, kind, gen) => {
                let id = goria
                    .write::<Map>()
                    .build_special_building(id, &obb, kind, gen);
                goria.write::<BuildingInfos>().insert(id);
            }
            SetGameTime(gt) => *goria.write::<GameTime>() = gt,
            MapBuildHouses => {
                let mut infos = goria.write::<BuildingInfos>();
                let mut map = goria.write::<Map>();
                for build in map.build_houses() {
                    infos.insert(build);
                }
            }
            MapLoadParis => {
                goria.write::<Map>().clear();
                map_model::procgen::load_parismap(&mut *goria.write::<Map>())
            }
            MapLoadTestField => {
                goria.write::<Map>().clear();
                map_model::procgen::load_testfield(&mut *goria.write::<Map>())
            }
            MapClear => goria.write::<Map>().clear(),
            MapGenerateTrees(aabb) => {
                goria.write::<Map>().trees.generate_chunks(aabb);
            }
        }
    }
}

impl std::iter::FromIterator<WorldCommands> for WorldCommands {
    fn from_iter<T: IntoIterator<Item = WorldCommands>>(iter: T) -> Self {
        Self {
            commands: iter.into_iter().flat_map(|x| x.commands).collect(),
        }
    }
}
