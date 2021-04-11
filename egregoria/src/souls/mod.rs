use crate::map_dynamic::BuildingInfos;
use crate::souls::goods_company::{company_soul, CompanyKind, GoodsCompany, GoodsCompanyRegistry};
use crate::souls::human::spawn_human;
use crate::vehicles::{spawn_parked_vehicle, VehicleKind};
use crate::Egregoria;
use common::FastMap;
use geom::Vec2;
use map_model::{BuildingID, BuildingKind};

#[macro_use]
pub mod desire;

pub mod goods_company;
pub mod human;

pub(crate) fn add_souls_to_empty_buildings(goria: &mut Egregoria) {
    let map = goria.map();
    let infos = goria.read::<BuildingInfos>();
    let mut empty_buildings: FastMap<BuildingKind, Vec<(BuildingID, Vec2)>> = FastMap::default();

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
        .take(100)
    {
        spawn_human(goria, build_id);
        n_souls_added += 1;
    }

    for (bkind, &(build_id, pos)) in empty_buildings
        .iter()
        .flat_map(|(bkind, v)| v.iter().map(move |x| (bkind, x)))
    {
        let registry = goria.read::<GoodsCompanyRegistry>();
        let des = &unwrap_or!(registry.descriptions.get(bkind), continue);

        let ckind = des.kind;
        let mk_trucks = |goria: &mut Egregoria| {
            let mut trucks = vec![];
            if let CompanyKind::Factory { n_trucks } = ckind {
                for _ in 0..n_trucks {
                    trucks.extend(spawn_parked_vehicle(goria, VehicleKind::Truck, pos))
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
                unwrap_or!(mk_trucks(goria), continue)
            },
        };

        company_soul(goria, comp);

        n_souls_added += 1;
    }

    if n_souls_added > 0 {
        log::info!("{} souls added", n_souls_added);
    }
}
