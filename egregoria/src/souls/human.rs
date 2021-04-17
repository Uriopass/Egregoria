use crate::economy::CommodityKind::JobOpening;
use crate::economy::{Bought, Market};
use crate::map_dynamic::{BuildingInfos, Destination, Router};
use crate::pedestrians::{spawn_pedestrian, Location};
use crate::souls::desire::{BuyFood, Home, Work};
use crate::utils::time::GameTime;
use crate::vehicles::{spawn_parked_vehicle, VehicleID, VehicleKind};
use crate::{Egregoria, ParCommandBuffer, SoulID};
use geom::Transform;
use legion::system;
use legion::Entity;
use map_model::BuildingID;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct HumanDecision {
    kind: HumanDecisionKind,
    wait: u8,
}

#[derive(Serialize, Deserialize)]
pub enum HumanDecisionKind {
    Yield,
    SetVehicle(Option<VehicleID>),
    GoTo(Destination),
    MultiStack(Vec<HumanDecisionKind>),
}

impl Default for HumanDecisionKind {
    fn default() -> Self {
        Self::Yield
    }
}

impl HumanDecisionKind {
    pub fn update(&mut self, router: &mut Router) -> bool {
        match *self {
            HumanDecisionKind::GoTo(dest) => router.go_to(dest),
            HumanDecisionKind::MultiStack(ref mut decisions) => {
                if let Some(d) = decisions.last_mut() {
                    if d.update(router) {
                        decisions.pop();
                    }
                    false
                } else {
                    true
                }
            }
            HumanDecisionKind::SetVehicle(id) => {
                router.use_vehicle(id);
                true
            }
            HumanDecisionKind::Yield => true,
        }
    }
}

#[derive(Debug)]
enum NextDesire<'a> {
    None,
    Home(&'a mut Home),
    Work(&'a mut Work),
    Food(&'a mut BuyFood),
}

register_system!(update_decision);
#[system(par_for_each)]
pub fn update_decision(
    #[resource] cbuf: &ParCommandBuffer,
    #[resource] time: &GameTime,
    #[resource] binfos: &BuildingInfos,
    me: &Entity,
    trans: &Transform,
    loc: &Location,
    router: &mut Router,
    bought: &mut Bought,
    decision: &mut HumanDecision,
    food: Option<&mut BuyFood>,
    home: Option<&mut Home>,
    work: Option<&mut Work>,
) {
    if decision.wait != 0 {
        decision.wait -= 1;
        return;
    }
    let pos = trans.position();
    decision.wait = (30.0 + common::rand::rand2(pos.x, pos.y) * 50.0) as u8;
    if !decision.kind.update(router) {
        return;
    }

    let soul = SoulID(*me);
    let mut decision_id = NextDesire::None;
    let mut max_score = f32::NEG_INFINITY;

    if let Some(home) = home {
        let score = home.score();

        if score > max_score {
            max_score = score;
            decision_id = NextDesire::Home(home);
        }
    }

    if let Some(work) = work {
        let score = work.score(time);

        if score > max_score {
            max_score = score;
            decision_id = NextDesire::Work(work);
        }
    }

    if let Some(food) = food {
        let score = food.score(time, loc, bought);

        #[allow(unused_assignments)]
        if score > max_score {
            max_score = score;
            decision_id = NextDesire::Food(food);
        }
    }

    match decision_id {
        NextDesire::Home(home) => decision.kind = home.apply(),
        NextDesire::Work(work) => decision.kind = work.apply(loc, router),
        NextDesire::Food(food) => {
            decision.kind = food.apply(cbuf, binfos, time, soul, trans, loc, bought)
        }
        NextDesire::None => {}
    }
}

pub fn spawn_human(goria: &mut Egregoria, house: BuildingID) -> Option<SoulID> {
    let map = goria.map();
    let housepos = map.buildings().get(house)?.door_pos;
    drop(map);

    let human = SoulID(spawn_pedestrian(goria, house)?);
    let car = spawn_parked_vehicle(goria, VehicleKind::Car, housepos);

    let mut m = goria.write::<Market>();
    m.buy(human, housepos, JobOpening, 1);
    drop(m);

    goria.write::<BuildingInfos>().set_owner(house, human);

    let time = goria.read::<GameTime>().instant();

    let mut e = goria.world.entry(human.0)?;

    e.add_component(HumanDecision::default());
    e.add_component(Home::new(house));
    e.add_component(BuyFood::new(time));
    e.add_component(Bought::default());
    e.add_component(Router::new(car));
    Some(human)
}
