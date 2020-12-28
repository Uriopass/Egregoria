use crate::map_dynamic::{Destination, Router};
use crate::souls::desire::Desire;
use common::{GameTime, RecTimeInterval, SECONDS_PER_HOUR};
use legion::system;
use map_model::BuildingID;

pub struct Home {
    house: BuildingID,
    home_inter: RecTimeInterval,
}

impl Home {
    pub fn new(house: BuildingID, offset: f32) -> Self {
        Home {
            house,
            home_inter: RecTimeInterval::new(
                (19, (offset * SECONDS_PER_HOUR as f32) as i32),
                (6, (offset * SECONDS_PER_HOUR as f32) as i32),
            ),
        }
    }
}

#[system(par_for_each)]
pub fn desire_home(#[resource] time: &GameTime, router: &mut Router, d: &mut Desire<Home>) {
    d.score_and_apply(
        |home| 1.0 - home.home_inter.dist_until(time.daytime) as f32 * 0.01,
        |home| {
            router.go_to(Destination::Building(home.house));
        },
    );
}
