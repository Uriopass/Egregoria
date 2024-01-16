use crate::map::BuildingKind;
use crate::map_dynamic::BuildingInfos;
use crate::souls::freight_station::freight_station_soul;
use crate::souls::goods_company::company_soul;
use crate::souls::human::spawn_human;
use crate::Simulation;

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
    let mut empty_buildings = Vec::with_capacity(16);

    for (id, building) in map.buildings() {
        if unwrap_cont!(infos.get(id)).owner.is_some() {
            continue;
        }

        empty_buildings.push((building.kind, id));
    }
    drop(infos);
    drop(map);

    let mut n_souls_added = 0;

    for (bkind, build_id) in empty_buildings {
        match bkind {
            BuildingKind::House => {
                spawn_human(sim, build_id);
                n_souls_added += 1;
            }
            BuildingKind::GoodsCompany(id) => {
                company_soul(sim, build_id, id);

                n_souls_added += 1;
            }
            BuildingKind::RailFreightStation(id) => {
                freight_station_soul(sim, build_id, id);
                n_souls_added += 1;
            }
            _ => {}
        }
    }

    if n_souls_added > 0 {
        log::info!("{} souls added", n_souls_added);
    }
}
