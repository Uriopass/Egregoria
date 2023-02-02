use crate::economy::Money;
use crate::engine_interaction::WorldCommand;
use crate::map::{LanePattern, MapProject};
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
            WorldCommand::AddTrain { n_wagons, .. } => 1000 + 100 * (*n_wagons as i64),
            WorldCommand::MapMakeConnection { from, to, pat, .. } => {
                Self::connection_cost(from, to, pat)
            }
            WorldCommand::UpdateZone {
                building: bid,
                zone: z,
            } => {
                let m = goria.map();
                let Some(b) = m.buildings.get(*bid) else { return Money::ZERO; };
                let Some(gc) = b.kind.as_goods_company() else { return Money::ZERO; };
                let registry = goria.read::<GoodsCompanyRegistry>();
                let zonedescr = registry.descriptions[gc].zone.as_ref().unwrap();

                let oldarea = b.zone.as_ref().map_or(0.0, |z| z.area);
                let newarea = z.area;
                return Money::new_base(
                    (newarea - oldarea) as i64 * zonedescr.price_per_area
                        / zonedescr.area_per_production,
                );
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
                    let descr = &goria.read::<GoodsCompanyRegistry>().descriptions[*x];
                    descr.price
                        + descr
                            .zone
                            .as_ref()
                            .map(|z| {
                                z.price_per_area * (descr.size * descr.size) as i64
                                    / z.area_per_production
                            })
                            .unwrap_or(0)
                }
                BuildingKind::RailFretStation => 1000,
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
