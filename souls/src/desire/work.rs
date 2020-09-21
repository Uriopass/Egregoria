use crate::desire::{Desire, Routed};
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

impl<T: Routed> Desire<T> for Work {
    fn score(&self, goria: &Egregoria, _soul: &T) -> f32 {
        (goria.read::<TimeInfo>().time / 500.0 + self.offset as f64).cos() as f32
    }

    fn apply(&mut self, goria: &Egregoria, soul: &mut T) -> Action {
        soul.router_mut()
            .go_to(goria, Destination::Building(self.workplace))
    }
}
