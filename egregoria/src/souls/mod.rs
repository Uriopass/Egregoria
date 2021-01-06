use crate::economy::{CommodityKind, Market};
use crate::map_dynamic::BuildingInfos;
use crate::souls::goods_company::{company_soul, CompanyKind, GoodsCompany, Recipe};
use crate::souls::human::spawn_human;
use crate::vehicles::{spawn_parked_vehicle, VehicleKind};
use crate::Egregoria;
use geom::Vec2;
use map_model::{BuildingID, BuildingKind, Map};
use rand::seq::SliceRandom;
use std::collections::HashMap;

#[macro_use]
pub mod desire;

pub mod goods_company;
pub mod human;

pub fn add_souls_to_empty_buildings(goria: &mut Egregoria) {
    let map = goria.read::<Map>();
    let infos = goria.read::<BuildingInfos>();
    let mut empty_buildings: HashMap<BuildingKind, Vec<(BuildingID, Vec2)>> = HashMap::new();

    for (id, building) in map.buildings() {
        if infos[id].owner.is_some() {
            continue;
        }

        empty_buildings
            .entry(building.kind)
            .or_default()
            .push((id, building.door_pos));
    }
    drop(infos);
    drop(map);

    let mut n_souls_added = 0;

    for &(build_id, _) in empty_buildings
        .get(&BuildingKind::House)
        .unwrap_or(&vec![])
        .choose_multiple(&mut rand::thread_rng(), 100)
    {
        spawn_human(goria, build_id);
        n_souls_added += 1;
    }

    for &(build_id, pos) in empty_buildings.get(&BuildingKind::Farm).unwrap_or(&vec![]) {
        let truck = unwrap_or!(
            spawn_parked_vehicle(goria, VehicleKind::Truck, pos),
            continue
        );

        let comp = GoodsCompany {
            kind: CompanyKind::Factory {
                truck,
                driver: None,
            },
            recipe: Recipe {
                consumption: &[],
                production: &[(CommodityKind::Wheat, 1)],
                seconds_per_work: 1000,
                storage_multiplier: 5,
            },
            building: build_id,
            workers: 10,
            progress: 0.0,
        };

        company_soul(goria, comp);
        n_souls_added += 1;
    }

    for &(build_id, pos) in empty_buildings
        .get(&BuildingKind::FlourFactory)
        .unwrap_or(&vec![])
    {
        let truck = unwrap_or!(
            spawn_parked_vehicle(goria, VehicleKind::Truck, pos),
            continue
        );

        let comp = GoodsCompany {
            kind: CompanyKind::Factory {
                truck,
                driver: None,
            },
            recipe: Recipe {
                consumption: &[(CommodityKind::Wheat, 1)],
                production: &[(CommodityKind::Flour, 1)],
                seconds_per_work: 1000,
                storage_multiplier: 2,
            },
            building: build_id,
            workers: 10,
            progress: 0.0,
        };

        company_soul(goria, comp);
        n_souls_added += 1;
    }

    for &(build_id, p) in empty_buildings
        .get(&BuildingKind::Bakery)
        .unwrap_or(&vec![])
    {
        let comp = GoodsCompany {
            kind: CompanyKind::Store,
            recipe: Recipe {
                consumption: &[(CommodityKind::Flour, 1)],
                production: &[(CommodityKind::Bread, 1)],
                seconds_per_work: 1000,
                storage_multiplier: 5,
            },
            building: build_id,
            workers: 3,
            progress: 0.0,
        };

        let s = company_soul(goria, comp);
        n_souls_added += 1;
    }

    if n_souls_added > 0 {
        log::info!("{} souls added", n_souls_added);
    }
}
