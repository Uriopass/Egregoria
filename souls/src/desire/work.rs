use crate::desire::Desire;
use crate::souls::human::Human;
use common::{GameTime, RecTimeInterval, SECONDS_PER_HOUR};
use egregoria::api::{Action, Destination};
use egregoria::Egregoria;
use map_model::BuildingID;

impl Work {
    pub fn new(workplace: BuildingID, offset: f32) -> Self {
        Work {
            workplace,
            work_inter: RecTimeInterval::new(
                (8, (offset * SECONDS_PER_HOUR as f32) as i32),
                (18, (offset * SECONDS_PER_HOUR as f32) as i32),
            ),
        }
    }
}

pub struct Work {
    workplace: BuildingID,
    work_inter: RecTimeInterval,
}

impl Desire<Human> for Work {
    fn name(&self) -> &'static str {
        "Work"
    }

    fn score(&self, goria: &Egregoria, _soul: &Human) -> f32 {
        let time = goria.read::<GameTime>();
        0.5 - self.work_inter.dist_until(time.daytime) as f32 * 0.01
    }

    fn apply(&mut self, goria: &Egregoria, soul: &mut Human) -> Action {
        soul.router
            .go_to(goria, Destination::Building(self.workplace))
    }
}
