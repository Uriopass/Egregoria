use crate::map::{LanePattern, MapProject, MAX_ZONE_AREA};
use crate::world_command::WorldCommand;
use crate::{BuildingKind, Simulation};
use prototypes::Money;
use serde::{Deserialize, Serialize};

/// The government represents the player.
#[derive(Serialize, Deserialize)]
pub struct Government {
    pub money: Money,
}

impl Default for Government {
    fn default() -> Self {
        Self {
            money: Money::new_bucks(150_000),
        }
    }
}

impl Government {
    pub fn action_cost(action: &WorldCommand, sim: &Simulation) -> Money {
        Money::new_bucks(match action {
            WorldCommand::MapBuildHouse(_) => 100,
            WorldCommand::AddTrain { n_wagons, .. } => 1000 + 100 * (*n_wagons as i64),
            WorldCommand::MapMakeConnection { from, to, pat, .. } => {
                Self::connection_cost(from, to, pat)
            }
            WorldCommand::UpdateZone {
                building: bid,
                zone: z,
            } => {
                let m = sim.map();
                let Some(b) = m.buildings.get(*bid) else {
                    log::error!("Trying to update zone of non-existent building");
                    return Money::ZERO;
                };
                let Some(gc) = b.kind.as_goods_company() else {
                    log::error!("Trying to update zone of non-goods-company building");
                    return Money::ZERO;
                };
                let Some(zonedescr) = gc.prototype().zone.as_ref() else {
                    log::error!("Trying to update zone of non-zoneable building");
                    return Money::ZERO;
                };

                let oldarea = b.zone.as_ref().map_or(0.0, |z| z.area);
                let newarea = z.area;
                return (newarea - oldarea) as i64 * zonedescr.price_per_area
                    / MAX_ZONE_AREA as i64;
            }
            WorldCommand::MapMakeMultipleConnections(ref projs, ref links) => {
                let mut total = 0;
                for (from, to, _, pat) in links.iter() {
                    total += Self::connection_cost(&projs[*from], &projs[*to], pat);
                }
                total
            }
            WorldCommand::MapBuildSpecialBuilding { kind: x, .. } => match x {
                BuildingKind::GoodsCompany(x) => {
                    let descr = x.prototype();
                    let mut price = descr.price;
                    if let Some(ref z) = descr.zone {
                        price += z.price_per_area * descr.size.area() as i64 / MAX_ZONE_AREA as i64;
                    }
                    return price;
                }
                BuildingKind::RailFreightStation(x) => {
                    return x.prototype().price;
                }
                BuildingKind::TrainStation => 1000,
                _ => 0,
            },
            _ => 0,
        })
    }

    fn connection_cost(p1: &MapProject, p2: &MapProject, pat: &LanePattern) -> i64 {
        let dist = p1.pos.distance(p2.pos);
        50 + ((0.03 * dist) as i64).max(1)
            * (pat.lanes_forward.len() + pat.lanes_backward.len()) as i64
    }
}
