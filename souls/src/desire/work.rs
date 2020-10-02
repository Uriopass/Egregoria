use crate::desire::Desire;
use crate::souls::human::Human;
use egregoria::api::{Action, Destination};
use egregoria::engine_interaction::TimeInfo;
use egregoria::Egregoria;
use map_model::BuildingID;

impl Work {
    pub fn new(workplace: BuildingID, offset: f32) -> Self {
        Work { workplace, offset }
    }
}

pub struct Work {
    workplace: BuildingID,
    offset: f32,
}

impl Desire<Human> for Work {
    fn name(&self) -> &'static str {
        "Work"
    }

    fn score(&self, goria: &Egregoria, _soul: &Human) -> f32 {
        ((goria.read::<TimeInfo>().time / 500.0) as f32 + self.offset).cos()
    }

    fn apply(&mut self, goria: &Egregoria, soul: &mut Human) -> Action {
        soul.router
            .go_to(goria, Destination::Building(self.workplace))
    }
}
