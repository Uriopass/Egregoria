use crate::map::{BuildingID, BuildingKind};
use crate::map_dynamic::BuildingInfos;
use crate::souls::freight_station::freight_station_soul;
use crate::souls::goods_company::{company_soul, GoodsCompany, GoodsCompanyRegistry};
use crate::souls::human::spawn_human;
use crate::transportation::{spawn_parked_vehicle, VehicleKind};
use crate::Simulation;
use geom::Vec3;
use prototypes::CompanyKind;
use std::collections::BTreeMap;

#[macro_use]
pub mod desire;

pub mod freight_station;
pub mod goods_company;
pub mod human;

/// Adds souls to empty buildings
pub(crate) fn add_souls_to_empty_buildings(sim: &mut Simulation) {
    profiling::scope!("souls::add_souls_to_empty_buildings");
    let map = sim.map();
    let infos = sim.read::<BuildingInfos>();
    let mut empty_buildings: BTreeMap<BuildingKind, Vec<(BuildingID, Vec3)>> = BTreeMap::default();

    for (id, building) in map.buildings() {
        if unwrap_cont!(infos.get(id)).owner.is_some() {
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
        .iter()
        .take(50)
    {
        spawn_human(sim, build_id);
        n_souls_added += 1;
    }

    for &(build_id, _) in empty_buildings
        .get(&BuildingKind::RailFreightStation)
        .unwrap_or(&vec![])
        .iter()
    {
        freight_station_soul(sim, build_id);
        n_souls_added += 1;
    }

    for (bkind, &(build_id, pos)) in empty_buildings
        .iter()
        .filter_map(|(kind, v)| kind.as_goods_company().zip(Some(v)))
        .flat_map(|(bkind, v)| v.iter().map(move |x| (bkind, x)))
    {
        let registry = sim.read::<GoodsCompanyRegistry>();
        let des = &unwrap_or!(registry.descriptions.get(bkind), continue);

        let ckind = des.kind;
        let mk_trucks = |sim: &mut Simulation| {
            let mut trucks = vec![];
            if let CompanyKind::Factory { n_trucks } = ckind {
                for _ in 0..n_trucks {
                    trucks.extend(spawn_parked_vehicle(sim, VehicleKind::Truck, pos))
                }
                if trucks.is_empty() {
                    return None;
                }
            }
            Some(trucks)
        };

        let comp = GoodsCompany {
            kind: des.kind,
            building: build_id,
            recipe: des.recipe.clone(),
            max_workers: des.n_workers,
            progress: 0.0,
            driver: None,
            trucks: {
                drop(registry);
                unwrap_or!(mk_trucks(sim), continue)
            },
        };

        company_soul(sim, comp);

        n_souls_added += 1;
    }

    if n_souls_added > 0 {
        log::info!("{} souls added", n_souls_added);
    }
}
