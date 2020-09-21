use crate::desire::Desire;
use egregoria::api::{Action, Location, PedestrianID};
use egregoria::engine_interaction::TimeInfo;
use egregoria::Egregoria;
use map_model::BuildingID;

impl Work {
    pub fn new(body: PedestrianID, workplace: BuildingID, offset: f32) -> Self {
        Work {
            body,
            workplace,
            offset,
        }
    }
}

pub struct Work {
    body: PedestrianID,
    workplace: BuildingID,
    offset: f32,
}

impl Desire for Work {
    fn score(&self, goria: &Egregoria) -> f32 {
        (goria.read::<TimeInfo>().time / 500.0 + self.offset as f64).cos() as f32
    }

    fn apply(&self, goria: &Egregoria) -> Action {
        Action::go_to(goria, self.body, Location::Building(self.workplace))
    }
}
