use crate::SoulID;
use legion::Entity;
use map_model::BuildingID;
use slotmap::SecondaryMap;

pub struct BuildingInfo {
    pub owners: Vec<SoulID>,
    pub inside: Vec<Entity>,
}

#[derive(Default)]
pub struct BuildingInfos {
    assignment: SecondaryMap<BuildingID, BuildingInfo>,
}

impl BuildingInfos {
    pub fn get_info_mut(&mut self, building: BuildingID) -> &mut BuildingInfo {
        if self.assignment.contains_key(building) {
            return self.assignment.get_mut(building).unwrap();
        }

        self.assignment.insert(
            building,
            BuildingInfo {
                owners: vec![],
                inside: vec![],
            },
        );
        self.assignment.get_mut(building).unwrap()
    }

    pub fn add_owner(&mut self, building: BuildingID, soul: SoulID) {
        self.get_info_mut(building).owners.push(soul);
    }

    pub fn get_in(&mut self, building: BuildingID, e: Entity) {
        if cfg!(debug_assertions) && self.get_info_mut(building).inside.contains(&e) {
            log::warn!(
                "called get_in({:?}, {:?}) but it was already inside",
                building,
                e
            );
        }
        self.get_info_mut(building).inside.push(e);
    }

    pub fn get_out(&mut self, building: BuildingID, e: Entity) {
        let inside = &mut self.get_info_mut(building).inside;
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
