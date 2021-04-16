use crate::map_dynamic::Destination;
use crate::souls::human::HumanDecisionKind;
use imgui_inspect_derive::*;
use map_model::BuildingID;
use serde::{Deserialize, Serialize};

#[derive(Inspect, Clone, Serialize, Deserialize, Debug)]
pub struct Home {
    house: BuildingID,
}

impl Home {
    pub fn new(house: BuildingID) -> Self {
        Home { house }
    }

    pub fn apply(&mut self) -> HumanDecisionKind {
        HumanDecisionKind::GoTo(Destination::Building(self.house))
    }

    pub fn score(&self) -> f32 {
        0.2
    }
}
