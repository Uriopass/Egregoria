use crate::desire::Desire;
use crate::human::Human;
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

impl Desire<Human> for Home {
    fn name(&self) -> &'static str {
        "Home"
    }

    fn score(&self, goria: &Egregoria, _soul: &Human) -> f32 {
        ((goria.read::<TimeInfo>().time / 500.0) as f32 + std::f32::consts::PI + self.offset).cos()
    }

    fn apply(&mut self, goria: &Egregoria, soul: &mut Human) -> Action {
        soul.router.go_to(goria, Destination::Building(self.house))
    }
}
