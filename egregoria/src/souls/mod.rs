use crate::map_dynamic::BuildingInfos;
use crate::souls::human::spawn_human;
use crate::Egregoria;
use map_model::{BuildingKind, Map};
use rand::seq::IteratorRandom;

#[macro_use]
pub mod desire;

pub mod human;
pub mod simple_producer;

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

    for &(build_id, _) in empty_buildings
        .iter()
        .filter(|(_, kind)| matches!(kind, BuildingKind::House))
        .choose_multiple(&mut rand::thread_rng(), 100)
    {
        if spawn_human(goria, build_id).is_some() {
            n_souls_added += 1;
        }
    }

    if n_souls_added > 0 {
        log::info!("{} souls added", n_souls_added);
    }
}
