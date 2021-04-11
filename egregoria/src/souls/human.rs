use crate::economy::CommodityKind::JobOpening;
use crate::economy::{Bought, Market};
use crate::map_dynamic::{BuildingInfos, Router};
use crate::pedestrians::{spawn_pedestrian, Pedestrian};
use crate::souls::desire::{BuyFood, Home, Work};
use crate::vehicles::{spawn_parked_vehicle, VehicleKind};
use crate::{Egregoria, SoulID};
use common::GameTime;
use map_model::BuildingID;

pub fn spawn_human(goria: &mut Egregoria, house: BuildingID) -> SoulID {
    let map = goria.map();
    let housepos = map.buildings()[house].door_pos;
    drop(map);

    let human = SoulID(spawn_pedestrian(goria, house));
    let car = spawn_parked_vehicle(goria, VehicleKind::Car, housepos);

    let mut m = goria.write::<Market>();
    m.buy(human, housepos, JobOpening, 1);
    drop(m);

    goria.write::<BuildingInfos>().set_owner(house, human);

    let time = goria.read::<GameTime>().instant();

    let mut e = goria.world.entry(human.0).unwrap();

    e.add_component(Desire::new(Home::new(house)));
    e.add_component(Desire::new(BuyFood::new(time)));
    e.add_component(Bought::default());
    e.add_component(Router::new(car));
    human
}

desires_system!(human_desires, Pedestrian, Home;0 Work;1 BuyFood;2);
