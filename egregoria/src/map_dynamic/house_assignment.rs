use crate::SoulID;
use map_model::BuildingID;
use serde::{Deserialize, Serialize};
use slotmap::SecondaryMap;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BuildingInfo {
    pub owner: Option<SoulID>,
    pub inside: Vec<SoulID>,
}

register_resource!(BuildingInfos, "binfos");
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct BuildingInfos {
    assignment: SecondaryMap<BuildingID, BuildingInfo>,
    owners: HashMap<SoulID, BuildingID>,
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

    pub fn building_owned_by(&self, soul: SoulID) -> Option<BuildingID> {
        self.owners.get(&soul).copied()
    }

    pub fn set_owner(&mut self, building: BuildingID, soul: SoulID) {
        if let Some(x) = self.get_mut(building) {
            x.owner = Some(soul)
        }
        self.owners.insert(soul, building);
    }

    pub fn get_in(&mut self, building: BuildingID, e: SoulID) {
        if cfg!(debug_assertions) && self[building].inside.contains(&e) {
            log::warn!(
                "called get_in({:?}, {:?}) but it was already inside",
                building,
                e
            );
        }
        self[building].inside.push(e);
    }

    pub fn get_out(&mut self, building: BuildingID, e: SoulID) {
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
