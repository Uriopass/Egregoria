use crate::economy::Money;
use crate::engine_interaction::WorldCommand;
use crate::{BuildingKind, Egregoria, GoodsCompanyRegistry};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Government {
    pub money: Money,
}

impl Default for Government {
    fn default() -> Self {
        Self {
            money: Money::new_base(150_000),
        }
    }
}

impl Government {
    pub fn action_cost(action: &WorldCommand, goria: &Egregoria) -> Money {
        Money::new_base(match action {
            WorldCommand::MapBuildHouse(_) => 100,
            WorldCommand::AddTrain(_, n_wagons, _) => 1000 + 100 * (*n_wagons as i64),
            WorldCommand::MapMakeConnection(p1, p2, _, pat) => {
                let dist = p1.pos.distance(p2.pos);
                50 + ((0.03 * dist) as i64).max(1)
                    * (pat.lanes_forward.len() + pat.lanes_backward.len()) as i64
            }
            WorldCommand::MapBuildSpecialBuilding(_, x, _) => match x {
                BuildingKind::GoodsCompany(x) => {
                    goria.read::<GoodsCompanyRegistry>().descriptions[*x].price
                }
                BuildingKind::RailFretStation => 1000,
                BuildingKind::TrainStation => 1000,
                _ => 0,
            },
            _ => 0,
        })
    }
}
