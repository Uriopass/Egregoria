use crate::economy::{Bought, Market};
use crate::map::BuildingID;
use crate::map_dynamic::{BuildingInfos, Destination, Itinerary, Router};
use crate::souls::desire::{BuyFood, Home, Work};
use crate::transportation::Speed;
use crate::transportation::{
    random_pedestrian_shirt_color, spawn_parked_vehicle, Location, Pedestrian, VehicleKind,
};
use crate::utils::rand_provider::RandProvider;
use crate::utils::resources::Resources;
use crate::world::{FreightStationEnt, HumanEnt, HumanID, VehicleID};
use crate::World;
use crate::{BuildingKind, Map, ParCommandBuffer, Simulation, SoulID};
use egui_inspect::Inspect;
use geom::Transform;
use lazy_static::lazy_static;
use prototypes::{GameTime, ItemID};
use serde::{Deserialize, Serialize};

#[derive(Inspect, Serialize, Deserialize, Default)]
pub struct HumanDecision {
    pub kind: HumanDecisionKind,
    pub wait: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HumanDecisionKind {
    Yield,
    SetVehicle(Option<VehicleID>),
    GoTo(Destination),
    DeliverAtBuilding(BuildingID),
    MultiStack(Vec<HumanDecisionKind>),
}

#[derive(Debug, Serialize, Deserialize, Inspect)]
pub enum Gender {
    M,
    F,
}

#[derive(Inspect, Serialize, Deserialize)]
pub struct PersonalInfo {
    pub name: String,
    pub age: u8,
    pub gender: Gender,
}

debug_inspect_impl!(HumanDecisionKind);

static FIRST_NAMES_BYTES: &str = include_str!("first_names.txt");
static LAST_NAMES_BYTES: &str = include_str!("names.txt");

lazy_static! {
    static ref LAST_NAMES: Vec<&'static str> = LAST_NAMES_BYTES.split('\n').collect();
    static ref FIRST_NAMES: Vec<&'static str> = FIRST_NAMES_BYTES.split('\n').collect();
}

impl PersonalInfo {
    pub fn new(rng: &mut RandProvider) -> Self {
        let age = (rng.next_f32() * 30.0 + 20.0) as u8;
        let gender = match rng.next_u32() % 2 {
            0 => Gender::M,
            1 => Gender::F,
            _ => unreachable!(),
        };

        let first_name = FIRST_NAMES[rng.next_u32() as usize % FIRST_NAMES.len()];
        let last_name = LAST_NAMES[rng.next_u32() as usize % LAST_NAMES.len()];

        let name = format!("{} {}", first_name, last_name);

        Self { name, age, gender }
    }
}

impl Default for HumanDecisionKind {
    fn default() -> Self {
        Self::Yield
    }
}

impl HumanDecisionKind {
    pub fn update(
        &mut self,
        router: &mut Router,
        binfos: &BuildingInfos,
        map: &Map,
        cbuf_freight: &ParCommandBuffer<FreightStationEnt>,
    ) -> bool {
        match *self {
            HumanDecisionKind::GoTo(dest) => router.go_to(dest),
            HumanDecisionKind::MultiStack(ref mut decisions) => {
                if let Some(d) = decisions.last_mut() {
                    if d.update(router, binfos, map, cbuf_freight) {
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
            HumanDecisionKind::DeliverAtBuilding(bid) => {
                let Some(b) = map.buildings().get(bid) else {
                    return true;
                };
                if matches!(b.kind, BuildingKind::RailFreightStation(_)) {
                    let Some(SoulID::FreightStation(fid)) = binfos.owner(bid) else {
                        return true;
                    };
                    cbuf_freight.exec_ent(fid, move |e| {
                        if let Some(f) = e.world.freight_stations.get_mut(fid) {
                            f.f.waiting_cargo += 1;
                        }
                    });
                }
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

pub fn update_decision_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("souls::update_decision_system");
    let ra = &*resources.read();
    let rb = &*resources.read();
    let rc = &*resources.read();
    let rd = &*resources.read();
    let re = &*resources.read();

    world.humans.iter_mut().for_each(|(ent, h)| {
        update_decision(
            ra,
            rb,
            rc,
            rd,
            re,
            ent,
            &h.trans,
            &h.location,
            &mut h.router,
            &mut h.bought,
            &mut h.decision,
            Some(&mut h.food),
            Some(&mut h.home),
            h.work.as_mut(),
        )
    });
}

#[allow(clippy::too_many_arguments)]
pub fn update_decision(
    cbuf: &ParCommandBuffer<HumanEnt>,
    cbuf_freight: &ParCommandBuffer<FreightStationEnt>,
    time: &GameTime,
    binfos: &BuildingInfos,
    map: &Map,
    me: HumanID,
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
    let pos = trans.pos;
    decision.wait = (30.0 + common::rand::rand2(pos.x, pos.y) * 50.0) as u8;
    if !decision.kind.update(router, binfos, map, cbuf_freight) {
        return;
    }

    let mut decision_id = NextDesire::None;
    let mut max_score = f32::NEG_INFINITY;

    if let Some(home) = home {
        let score = home.score();
        home.last_score = score;

        if score > max_score {
            max_score = score;
            decision_id = NextDesire::Home(home);
        }
    }

    if let Some(work) = work {
        let score = work.score(time);
        work.last_score = score;

        if score > max_score {
            max_score = score;
            decision_id = NextDesire::Work(work);
        }
    }

    if let Some(food) = food {
        let score = food.score(time, loc, bought);
        food.last_score = score;

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
            decision.kind = food.apply(cbuf, binfos, time, me, trans, loc, bought)
        }
        NextDesire::None => {}
    }
}

pub fn spawn_human(sim: &mut Simulation, house: BuildingID) -> Option<HumanID> {
    profiling::scope!("spawn_human");
    let map = sim.map();
    let housepos = map.buildings().get(house)?.door_pos;
    drop(map);

    let _color = random_pedestrian_shirt_color(&mut sim.write::<RandProvider>());

    let hpos = sim.map().buildings().get(house)?.door_pos;
    let p = Pedestrian::new(&mut sim.write::<RandProvider>());

    let time = sim.read::<GameTime>().instant();

    let car = spawn_parked_vehicle(sim, VehicleKind::Car, housepos);

    let personal_info = Box::new(PersonalInfo::new(&mut sim.write::<RandProvider>()));

    let id = sim.world.insert(HumanEnt {
        trans: Transform::new(hpos),
        location: Location::Building(house),
        pedestrian: p,
        it: Itinerary::NONE,
        speed: Speed::default(),
        decision: HumanDecision::default(),
        home: Home::new(house),
        food: BuyFood::new(time),
        bought: Bought::default(),
        router: Router::new(car),
        collider: None,
        work: None,
        personal_info,
    });

    let soul = SoulID::Human(id);
    let mut m = sim.write::<Market>();
    m.buy(soul, housepos.xy(), ItemID::new("job-opening"), 1);

    sim.write::<BuildingInfos>().get_in(house, soul);
    sim.write::<BuildingInfos>().set_owner(house, soul);

    Some(id)
}
