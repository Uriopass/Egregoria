use crate::map::BuildingID;
use crate::SoulID;
use serde::{Deserialize, Serialize};
use slotmapd::SecondaryMap;
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct BuildingInfo {
    pub owner: Option<SoulID>,
    pub inside: Vec<SoulID>,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct BuildingInfos {
    assignment: SecondaryMap<BuildingID, BuildingInfo>,
    owners: BTreeMap<SoulID, BuildingID>,
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

    pub fn owner(&self, building: BuildingID) -> Option<SoulID> {
        self.assignment.get(building).and_then(|x| x.owner)
    }

    pub fn get_in(&mut self, building: BuildingID, e: SoulID) {
        let b = unwrap_ret!(self.get_mut(building));
        if cfg!(debug_assertions) && b.inside.contains(&e) {
            log::warn!(
                "called get_in({:?}, {:?}) but it was already inside",
                building,
                e
            );
        }
        b.inside.push(e);
    }

    pub fn get_out(&mut self, building: BuildingID, e: SoulID) {
        let b = unwrap_ret!(self.get_mut(building));
        let inside = &mut b.inside;
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
