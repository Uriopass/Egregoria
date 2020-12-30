use crate::economy::{JobApplication, Market};
use crate::map_dynamic::{BuildingInfos, Router};
use crate::pedestrians::spawn_pedestrian;
use crate::souls::desire::{Home, Work};
use crate::vehicles::spawn_parked_vehicle;
use crate::{Egregoria, SoulID};
use map_model::{BuildingID, Map};

pub fn spawn_human(goria: &mut Egregoria, house: BuildingID) -> Option<()> {
    let map = goria.read::<Map>();
    let housepos = map.buildings()[house].door_pos;
    drop(map);

    let human = SoulID(spawn_pedestrian(goria, house));
    let car = spawn_parked_vehicle(goria, housepos);

    goria
        .write::<Market<JobApplication>>()
        .buy(human, housepos, 1);
    goria.write::<BuildingInfos>().set_owner(house, human);

    let mut e = goria.world.entry(human.0).unwrap();

    e.add_component(Desire::new(Home::new(house)));
    e.add_component(Router::new(car));
    Some(())
}

desires_system!(human_desires, Home;0 Work;1);
