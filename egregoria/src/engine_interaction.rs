use crate::{ent_from_id, ent_id, Egregoria};
use map_model::{
    BuildingGen, BuildingID, BuildingKind, IntersectionID, LanePattern, LightPolicy, LotID, Map,
    MapProject, RoadID, TurnPolicy,
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

#[derive(Clone, Default)]
pub struct WorldCommands {
    pub(crate) commands: Vec<WorldCommand>,
}

defer_serialize!(WorldCommands, Vec<WorldCommand>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldCommand {
    MapRemoveIntersection(IntersectionID),
    MapRemoveRoad(RoadID),
    MapRemoveBuilding(BuildingID),
    MapBuildHouse(LotID),
    MapMakeConnection(MapProject, MapProject, Option<Vec2>, LanePattern),
    MapUpdateIntersectionPolicy(IntersectionID, TurnPolicy, LightPolicy),
    MapBuildSpecialBuilding(RoadID, OBB, BuildingKind, BuildingGen),
    MapLoadParis,
    MapLoadTestField(Vec2, u32, f32),
    ResetSave,
    SetGameTime(GameTime),
    UpdateTransform(u64, Transform),
}

use crate::economy::Government;
use crate::map_dynamic::BuildingInfos;
use crate::utils::time::GameTime;
use geom::{Transform, Vec2, OBB};
use legion::Entity;
use WorldCommand::*;

impl WorldCommands {
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
        self.commands.push(MapLoadTestField(pos, size, spacing))
    }

    pub fn update_transform(&mut self, e: Entity, trans: Transform) {
        self.commands.push(UpdateTransform(ent_id(e), trans))
    }

    pub fn reset_save(&mut self) {
        self.commands.push(ResetSave)
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
        goria.write::<Government>().money.0 -= cost.0;
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
            MapBuildSpecialBuilding(id, obb, kind, gen) => {
                if let Some(id) = goria
                    .write::<Map>()
                    .build_special_building(id, &obb, kind, gen)
                {
                    goria.write::<BuildingInfos>().insert(id);
                }
            }
            SetGameTime(gt) => *goria.write::<GameTime>() = gt,
            MapLoadParis => map_model::procgen::load_parismap(&mut *goria.map_mut()),
            MapLoadTestField(pos, size, spacing) => {
                map_model::procgen::load_testfield(&mut *goria.map_mut(), pos, size, spacing)
            }
            ResetSave => {
                *goria = Egregoria::new(10);
            }
            UpdateTransform(e, t) => {
                if let Some(x) = goria.comp_mut(ent_from_id(e)) {
                    *x = t
                }
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
