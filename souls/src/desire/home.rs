use crate::desire::Desire;
use egregoria::api::{Action, Location};
use egregoria::engine_interaction::TimeInfo;
use egregoria::Egregoria;
use legion::Entity;
use map_model::BuildingID;

pub struct Home {
    body: Entity,
    house: BuildingID,
    offset: f32,
}

impl Home {
    pub fn new(body: Entity, house: BuildingID, offset: f32) -> Self {
        Home {
            body,
            house,
            offset,
        }
    }
}

impl Desire for Home {
    fn score(&self, goria: &Egregoria) -> f32 {
        (goria.read::<TimeInfo>().time / 100.0 + std::f64::consts::PI + self.offset as f64).cos()
            as f32
    }

    fn apply(&self, _goria: &Egregoria) -> Action {
        Action::WalkTo(self.body, Location::Building(self.house))
    }
}
