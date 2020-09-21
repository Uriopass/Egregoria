use crate::desire::{Home, Routed, Work};
use crate::souls::Soul;
use egregoria::api::Router;
use egregoria::map_dynamic::BuildingInfos;
use egregoria::pedestrians::spawn_pedestrian;
use egregoria::utils::rand_provider::RandProvider;
use egregoria::{Egregoria, SoulID};
use map_model::{BuildingID, BuildingKind, Map};

pub struct Human {
    router: Router,
}

impl Routed for Human {
    fn router_mut(&mut self) -> &mut Router {
        &mut self.router
    }
}

impl Human {
    pub fn soul(id: SoulID, house: BuildingID, goria: &mut Egregoria) -> Option<Soul<Human>> {
        let map = goria.read::<Map>();
        let work = map
            .random_building(BuildingKind::Workplace, &mut *goria.write::<RandProvider>())?
            .id;
        drop(map);

        goria.write::<BuildingInfos>().add_owner(house, id);

        let body = spawn_pedestrian(goria, house);

        let offset = goria.write::<RandProvider>().random::<f32>() * 0.5;

        let router = Router::new(body);

        Some(Soul {
            id,
            desires: vec![
                Box::new(Work::new(work, offset)),
                Box::new(Home::new(house, offset)),
            ],
            extra: Human { router },
        })
    }
}
