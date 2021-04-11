use crate::map_dynamic::{Destination, Router};
use crate::souls::desire::Desire;
use imgui_inspect_derive::*;
use legion::system;
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
}

register_system!(desire_home);
#[system(par_for_each)]
pub fn desire_home(router: &mut Router, d: &mut Desire<Home>) {
    d.score_and_apply(
        |_| 0.2,
        |home| {
            router.go_to(Destination::Building(home.house));
        },
    );
}
