use crate::desire::{Desire, Routed};
use egregoria::api::{Action, Destination};
use egregoria::engine_interaction::TimeInfo;
use egregoria::Egregoria;
use map_model::BuildingID;

pub struct Home {
    house: BuildingID,
    offset: f32,
}

impl Home {
    pub fn new(house: BuildingID, offset: f32) -> Self {
        Home { house, offset }
    }
}

impl<T: Routed> Desire<T> for Home {
    fn name(&self) -> &'static str {
        "Home"
    }

    fn score(&self, goria: &Egregoria, _soul: &T) -> f32 {
        (goria.read::<TimeInfo>().time / 500.0 + std::f64::consts::PI + self.offset as f64).cos()
            as f32
    }

    fn apply(&mut self, goria: &Egregoria, soul: &mut T) -> Action {
        soul.router_mut()
            .go_to(goria, Destination::Building(self.house))
    }
}
