use crate::desire::Desire;
use egregoria::api::{Action, Location};
use egregoria::engine_interaction::TimeInfo;
use egregoria::Egregoria;
use legion::Entity;
use map_model::BuildingID;

impl Work {
    pub fn new(body: Entity, workplace: BuildingID, offset: f32) -> Self {
        Work {
            body,
            workplace,
            offset,
        }
    }
}

pub struct Work {
    body: Entity,
    workplace: BuildingID,
    offset: f32,
}

impl Desire for Work {
    fn score(&self, goria: &Egregoria) -> f32 {
        (goria.read::<TimeInfo>().time / 500.0 + self.offset as f64).cos() as f32
    }

    fn apply(&self, goria: &Egregoria) -> Action {
        Action::walk_to(goria, self.body, Location::Building(self.workplace))
    }
}
