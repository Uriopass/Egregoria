use crate::economy::CommodityKind;
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

pub struct GoodsCompanyDescription {
    pub name: &'static str,
    pub bkind: BuildingKind,
    pub kind: CompanyKind,
    pub recipe: Recipe,
    pub n_workers: i32,
    pub size: f32,
    pub asset_location: &'static str,
}

pub const GOODS_BUILDINGSS: &[GoodsCompanyDescription] = &[
    GoodsCompanyDescription {
        name: "Cereal Farm",
        bkind: BuildingKind::CerealFarm,
        kind: CompanyKind::Factory { n_trucks: 1 },
        recipe: Recipe {
            consumption: &[],
            production: &[(CommodityKind::Cereal, 1)],
            seconds_per_work: 1000,
            storage_multiplier: 5,
        },
        n_workers: 10,
        size: 80.0,
        asset_location: "assets/cereal_farm.png",
    },
    GoodsCompanyDescription {
        name: "Cereal Factory",
        bkind: BuildingKind::CerealFactory,
        kind: CompanyKind::Factory { n_trucks: 1 },
        recipe: Recipe {
            consumption: &[(CommodityKind::Cereal, 1)],
            production: &[(CommodityKind::Flour, 1)],
            seconds_per_work: 1000,
            storage_multiplier: 2,
        },
        n_workers: 10,
        size: 80.0,
        asset_location: "assets/flour_factory.png",
    },
    GoodsCompanyDescription {
        name: "Bakery",
        bkind: BuildingKind::Bakery,
        kind: CompanyKind::Store,
        recipe: Recipe {
            consumption: &[(CommodityKind::Flour, 1)],
            production: &[(CommodityKind::Bread, 1)],
            seconds_per_work: 1000,
            storage_multiplier: 5,
        },
        n_workers: 3,
        size: 10.0,
        asset_location: "assets/bakery.png",
    },
];

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

    for des in GOODS_BUILDINGSS {
        for &(build_id, pos) in empty_buildings.get(&des.bkind).unwrap_or(&vec![]) {
            let mut trucks = vec![];

            if let CompanyKind::Factory { n_trucks } = des.kind {
                for _ in 0..n_trucks {
                    trucks.extend(spawn_parked_vehicle(goria, VehicleKind::Truck, pos))
                }
                if trucks.is_empty() {
                    continue;
                }
            }

            let comp = GoodsCompany {
                kind: des.kind,
                building: build_id,
                recipe: des.recipe,
                workers: des.n_workers,
                progress: 0.0,
                driver: None,
                trucks,
            };

            company_soul(goria, comp);

            n_souls_added += 1;
        }
    }

    if n_souls_added > 0 {
        log::info!("{} souls added", n_souls_added);
    }
}
