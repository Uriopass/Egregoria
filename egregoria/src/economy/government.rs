use crate::economy::Money;
use crate::engine_interaction::WorldCommand;
use crate::Egregoria;
use serde::{Deserialize, Serialize};

register_resource!(Government, "government");
#[derive(Serialize, Deserialize)]
pub struct Government {
    pub money: Money,
}

impl Default for Government {
    fn default() -> Self {
        Self {
            money: Money::base(10_000),
        }
    }
}

impl Government {
    pub fn action_cost(_action: &WorldCommand, _goria: &Egregoria) -> Money {
        Money(100)
    }
}
