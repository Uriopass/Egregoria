use crate::economy::{Bought, ItemRegistry, Market};
use crate::map::BuildingID;
use crate::map_dynamic::{BuildingInfos, Destination, Router};
use crate::pedestrians::{spawn_pedestrian, Location};
use crate::souls::desire::{BuyFood, Home, Work};
use crate::utils::time::GameTime;
use crate::vehicles::{spawn_parked_vehicle, VehicleID, VehicleKind};
use crate::{Egregoria, Map, ParCommandBuffer, SoulID};
use geom::Transform;
use hecs::{Entity, World};
use imgui_inspect_derive::Inspect;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use resources::Resources;
use serde::{Deserialize, Serialize};

#[derive(Inspect, Serialize, Deserialize, Default)]
pub struct HumanDecision {
    kind: HumanDecisionKind,
    wait: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HumanDecisionKind {
    Yield,
    SetVehicle(Option<VehicleID>),
    GoTo(Destination),
    MultiStack(Vec<HumanDecisionKind>),
}

debug_inspect_impl!(HumanDecisionKind);

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

#[profiling::function]
pub fn update_decision_system(world: &mut World, resources: &mut Resources) {
    let ra = &*resources.get().unwrap();
    let rb = &*resources.get().unwrap();
    let rc = &*resources.get().unwrap();
    let rd = &*resources.get().unwrap();
    world
        .query::<(
            &Transform,
            &Location,
            &mut Router,
            &mut Bought,
            &mut HumanDecision,
            Option<&mut BuyFood>,
            Option<&mut Home>,
            Option<&mut Work>,
        )>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|batch| {
            batch.for_each(|(ent, (a, b, c, d, e, f, g, h))| {
                update_decision(ra, rb, rc, rd, ent, a, b, c, d, e, f, g, h);
            })
        })
}

#[allow(clippy::too_many_arguments)]
pub fn update_decision(
    cbuf: &ParCommandBuffer,
    time: &GameTime,
    binfos: &BuildingInfos,
    map: &Map,
    me: Entity,
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
    let pos = trans.position;
    decision.wait = (30.0 + common::rand::rand2(pos.x, pos.y) * 50.0) as u8;
    if !decision.kind.update(router) {
        return;
    }

    let soul = SoulID(me);
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
            decision.kind = food.apply(cbuf, binfos, map, time, soul, trans, loc, bought)
        }
        NextDesire::None => {}
    }
}

#[profiling::function]
pub fn spawn_human(goria: &mut Egregoria, house: BuildingID) -> Option<SoulID> {
    let map = goria.map();
    let housepos = map.buildings().get(house)?.door_pos;
    drop(map);

    let human = SoulID(spawn_pedestrian(goria, house)?);
    let car = spawn_parked_vehicle(goria, VehicleKind::Car, housepos);

    let mut m = goria.write::<Market>();
    {}
    let registry = goria.read::<ItemRegistry>();
    m.buy(human, housepos.xy(), registry.id("job-opening"), 1);
    drop(m);

    goria.write::<BuildingInfos>().set_owner(house, human);

    let time = goria.read::<GameTime>().instant();

    let food = BuyFood::new(time, &*registry);
    drop(registry);

    goria
        .world
        .insert(
            human.0,
            (
                HumanDecision::default(),
                Home::new(house),
                food,
                Bought::default(),
                Router::new(car),
            ),
        )
        .unwrap();
    Some(human)
}
