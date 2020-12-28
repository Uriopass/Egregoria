use crate::map_dynamic::BuildingInfos;
use crate::souls::human::Human;
use crate::Egregoria;
use map_model::{BuildingKind, Map};

#[macro_use]
pub mod desire;

pub mod human;
pub mod supermarket;

pub fn add_souls_to_empty_buildings(goria: &mut Egregoria) {
    let map = goria.read::<Map>();
    let infos = goria.read::<BuildingInfos>();
    let mut empty_buildings = vec![];
    for (id, building) in map.buildings() {
        if !matches!(building.kind, BuildingKind::House) {
            continue;
        }
        if infos[id].owner.is_none() {
            empty_buildings.push((id, building.kind));
        }
    }
    drop(infos);
    drop(map);

    let mut n_souls_added = 0;

    for (build_id, kind) in empty_buildings {
        match kind {
            BuildingKind::House => {
                if Human::soul(goria, build_id).is_some() {
                    n_souls_added += 1;
                }
            }
            _ => unreachable!(),
        }

        if n_souls_added > 100 {
            break;
        }
    }

    if n_souls_added > 0 {
        log::info!("{} souls added", n_souls_added);
    }
}
