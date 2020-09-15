use crate::desire::Desire;
use egregoria::api::{Action, Location};
use egregoria::engine_interaction::TimeInfo;
use egregoria::Egregoria;
use legion::Entity;
use map_model::BuildingID;

pub struct Work {
    body: Entity,
    workplace: BuildingID,
    offset: f32,
}

impl Work {
    pub fn new(body: Entity, workplace: BuildingID, offset: f32) -> Self {
        Work {
            body,
            workplace,
            offset,
        }
    }
}

impl Desire for Work {
    fn score(&self, goria: &Egregoria) -> f32 {
        (goria.read::<TimeInfo>().time / 100.0 + self.offset as f64).cos() as f32
    }

    fn apply(&self, _goria: &Egregoria) -> Action {
        Action::WalkTo(self.body, Location::Building(self.workplace))
    }
}
