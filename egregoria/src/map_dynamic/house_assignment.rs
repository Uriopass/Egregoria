use crate::pedestrians::data::PedestrianID;
use crate::SoulID;
use map_model::BuildingID;
use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;
use std::ops::{Index, IndexMut};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BuildingInfo {
    pub owner: Option<SoulID>,
    pub inside: Vec<PedestrianID>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BuildingInfos {
    assignment: SecondaryMap<BuildingID, BuildingInfo>,
}

impl BuildingInfos {
    pub fn insert(&mut self, building: BuildingID) {
        self.assignment.insert(building, BuildingInfo::default());
    }

    pub fn get(&self, building: BuildingID) -> Option<&BuildingInfo> {
        self.assignment.get(building)
    }

    pub fn get_mut(&mut self, building: BuildingID) -> Option<&mut BuildingInfo> {
        self.assignment.get_mut(building)
    }

    pub fn set_owner(&mut self, building: BuildingID, soul: SoulID) {
        if let Some(x) = self.get_mut(building) {
            x.owner = Some(soul)
        }
    }

    pub fn get_in(&mut self, building: BuildingID, e: PedestrianID) {
        if cfg!(debug_assertions) && self[building].inside.contains(&e) {
            log::warn!(
                "called get_in({:?}, {:?}) but it was already inside",
                building,
                e
            );
        }
        self[building].inside.push(e);
    }

    pub fn get_out(&mut self, building: BuildingID, e: PedestrianID) {
        let inside = &mut self[building].inside;
        if let Some(i) = inside.iter().position(|v| *v == e) {
            inside.swap_remove(i);
        } else {
            log::warn!(
                "called get_out({:?}, {:?}) but it was not inside",
                building,
                e
            );
        }
    }
}

impl Index<BuildingID> for BuildingInfos {
    type Output = BuildingInfo;

    fn index(&self, index: BuildingID) -> &Self::Output {
        &self.assignment[index]
    }
}

impl IndexMut<BuildingID> for BuildingInfos {
    fn index_mut(&mut self, index: BuildingID) -> &mut Self::Output {
        &mut self.assignment[index]
    }
}
