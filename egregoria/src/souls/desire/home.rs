use crate::map_dynamic::{Destination, Router};
use crate::souls::desire::Desire;
use legion::system;
use map_model::BuildingID;

pub struct Home {
    house: BuildingID,
}

impl Home {
    pub fn new(house: BuildingID) -> Self {
        Home { house }
    }
}

#[system(par_for_each)]
pub fn desire_home(router: &mut Router, d: &mut Desire<Home>) {
    d.score_and_apply(
        |home| 0.2,
        |home| {
            router.go_to(Destination::Building(home.house));
        },
    );
}
