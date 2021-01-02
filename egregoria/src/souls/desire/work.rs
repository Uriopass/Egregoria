use crate::map_dynamic::{Destination, Router};
use crate::souls::desire::Desire;
use common::{GameTime, RecTimeInterval, SECONDS_PER_HOUR};
use legion::system;
use map_model::BuildingID;

pub struct Work {
    pub(crate) workplace: BuildingID,
    work_inter: RecTimeInterval,
}

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

#[system(par_for_each)]
pub fn desire_work(#[resource] time: &GameTime, router: &mut Router, d: &mut Desire<Work>) {
    d.score_and_apply(
        |work| {
            0.5 /*
                if work.work_inter.dist_until(time.daytime) == 0 {
                    0.5
                } else {
                    0.0
                }*/
        },
        |work| {
            router.go_to(Destination::Building(work.workplace));
        },
    )
}
