use crate::map_dynamic::BuildingInfos;
use crate::souls::goods_company::{company_soul, CompanyKind, GoodsCompany, GOODS_BUILDINGS};
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

    for des in GOODS_BUILDINGS {
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
                work_seconds: 0.0,
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
