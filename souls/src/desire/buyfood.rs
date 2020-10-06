use crate::desire::Desire;
use crate::souls::human::Human;
use egregoria::api::{Action, Destination};
use egregoria::economy::{Goods, Market};
use egregoria::map_dynamic::BuildingInfos;
use egregoria::Egregoria;
use geom::Vec2;
use map_model::{BuildingID, BuildingKind, Map};
use ordered_float::OrderedFloat;

pub struct BuyFood {
    min_level: i32,
    supermarket: Option<BuildingID>,
}

impl BuyFood {
    pub fn new(min_level: i32) -> Self {
        BuyFood {
            min_level,
            supermarket: None,
        }
    }
}

impl Desire<Human> for BuyFood {
    fn name(&self) -> &'static str {
        "Buy food"
    }

    fn score(&self, goria: &Egregoria, soul: &Human) -> f32 {
        if goria.read::<Market>().agents[&soul.id].goods.food < self.min_level {
            0.8
        } else {
            -100.0
        }
    }

    fn apply(&mut self, goria: &Egregoria, soul: &mut Human) -> Action {
        if self.supermarket.is_none() {
            self.supermarket = find_supermarket(soul.router.body_pos(goria), &goria.read::<Map>());
        }

        if let Some(id) = self.supermarket {
            if soul.router.arrived(Destination::Building(id)) {
                if let Some(owner) = goria.read::<BuildingInfos>()[id].owner {
                    let trans = goria.read::<Market>().want(owner, Goods { food: 1 });
                    if trans.is_none() {
                        log::warn!("No food in this supermarket :( {:?}", id);
                        return Action::DoNothing;
                    }

                    let trans = trans.unwrap();
                    return Action::Buy {
                        buyer: soul.id,
                        seller: owner,
                        trans,
                    };
                }
            }

            soul.router.go_to(goria, Destination::Building(id))
        } else {
            Action::DoNothing
        }
    }
}

fn find_supermarket(pos: Vec2, map: &Map) -> Option<BuildingID> {
    map.buildings()
        .values()
        .filter(|x| matches!(x.kind, BuildingKind::Supermarket))
        .min_by_key(|b| OrderedFloat(b.door_pos.distance2(pos)))
        .map(|x| x.id)
}
