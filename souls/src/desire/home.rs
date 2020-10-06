use crate::desire::Desire;
use crate::human::Human;
use common::{GameTime, RecTimeInterval, SECONDS_PER_HOUR};
use egregoria::api::{Action, Destination};
use egregoria::Egregoria;
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
                (7, 00),
            ),
        }
    }
}

impl Desire<Human> for Home {
    fn name(&self) -> &'static str {
        "Home"
    }

    fn score(&self, goria: &Egregoria, _soul: &Human) -> f32 {
        let time = goria.read::<GameTime>();
        0.5 - self.home_inter.dist_until(time.daytime) as f32 * 0.01
    }

    fn apply(&mut self, goria: &Egregoria, soul: &mut Human) -> Action {
        soul.router.go_to(goria, Destination::Building(self.house))
    }
}
