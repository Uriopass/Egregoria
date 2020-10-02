use crate::desire::{BuyFood, Home, Work};
use crate::souls::Soul;
use egregoria::api::Router;
use egregoria::economy::{EconomicAgent, Goods, Market, Money};
use egregoria::map_dynamic::BuildingInfos;
use egregoria::pedestrians::spawn_pedestrian;
use egregoria::utils::rand_provider::RandProvider;
use egregoria::vehicles::spawn_parked_vehicle;
use egregoria::{Egregoria, SoulID};
use map_model::{BuildingID, BuildingKind, Map};

pub type HumanSoul = Soul<Human, (Work, Home, BuyFood)>;

pub struct Human {
    pub id: SoulID,
    pub router: Router,
}

impl Human {
    pub fn soul(goria: &mut Egregoria, id: SoulID, house: BuildingID) -> Option<HumanSoul> {
        let map = goria.read::<Map>();
        let work = map
            .random_building(BuildingKind::Workplace, &mut *goria.write::<RandProvider>())?
            .id;
        let housepos = map.buildings()[house].door_pos;
        drop(map);

        goria.write::<BuildingInfos>().set_owner(house, id);

        let body = spawn_pedestrian(goria, house);
        let car = spawn_parked_vehicle(goria, housepos);

        let offset = goria.write::<RandProvider>().random::<f32>() * 0.5;

        let router = Router::new(body, car);

        goria
            .write::<Market>()
            .agents
            .insert(id, EconomicAgent::new(id, Money(10000), Goods { food: 0 }));

        Some(Soul {
            desires: (
                Work::new(work, offset),
                Home::new(house, offset),
                BuyFood::new(7),
            ),
            extra: Human { id, router },
        })
    }
}
