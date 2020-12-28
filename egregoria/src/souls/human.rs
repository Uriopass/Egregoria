use crate::map_dynamic::{BuildingInfos, Router};
use crate::pedestrians::spawn_pedestrian;
use crate::souls::desire::{Home, Work};
use crate::utils::rand_provider::RandProvider;
use crate::vehicles::spawn_parked_vehicle;
use crate::{Egregoria, SoulID};
use map_model::{BuildingID, BuildingKind, Map};

pub struct Human {
    pub id: SoulID,
    pub router: Router,
}

impl Human {
    pub fn soul(goria: &mut Egregoria, house: BuildingID) -> Option<()> {
        let map = goria.read::<Map>();
        let work = map
            .random_building(BuildingKind::Workplace, &mut *goria.write::<RandProvider>())?
            .id;
        let housepos = map.buildings()[house].door_pos;
        drop(map);

        let human = SoulID(spawn_pedestrian(goria, house));
        let car = spawn_parked_vehicle(goria, housepos);

        goria.write::<BuildingInfos>().set_owner(house, human);

        let offset = goria.write::<RandProvider>().random::<f32>() * 0.5;
        let mut e = goria.world.entry(human.0).unwrap();

        e.add_component(Desire::new(Work::new(work, offset)));
        e.add_component(Desire::new(Home::new(house, offset)));
        e.add_component(Router::new(car));
        Some(())
    }
}

desires_system!(human_desires, Home;0 Work;1);
