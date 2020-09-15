use crate::desire::{Desire, Home, Work};
use egregoria::api::Action;
use egregoria::map_dynamic::BuildingInfos;
use egregoria::pedestrians::spawn_pedestrian;
use egregoria::utils::rand_provider::RandProvider;
use egregoria::{Egregoria, SoulID};
use map_model::{BuildingID, BuildingKind, Map};
use ordered_float::OrderedFloat;
use slotmap::DenseSlotMap;

mod desire;

#[derive(Default)]
pub struct Souls {
    pub souls: DenseSlotMap<SoulID, Soul>,
}

impl Souls {
    pub fn add_souls_to_empty_buildings(&mut self, goria: &mut Egregoria) {
        let map = goria.read::<Map>();
        let mut infos = goria.write::<BuildingInfos>();
        let mut empty_buildings = vec![];
        for (id, building) in map.buildings() {
            if building.kind != BuildingKind::House {
                continue;
            }
            if infos.get_info_mut(id).owners.is_empty() {
                empty_buildings.push(id);
            }
        }
        drop(map);
        drop(infos);

        for house in empty_buildings {
            let id = self.souls.insert_with_key(Soul::empty);

            if let Some(soul) = Soul::human_soul(id, house, goria) {
                self.souls[id] = soul;
            } else {
                self.souls.remove(id);
            }
        }
    }

    pub fn update(&mut self, goria: &mut Egregoria) {
        let actions: Vec<_> = self
            .souls
            .iter_mut()
            .map(|(_, x)| x.decision(goria))
            .collect();
        for action in actions {
            let _ = action.apply(goria);
        }
    }
}

pub struct Soul {
    pub id: SoulID,
    desires: Vec<Box<dyn Desire>>,
}

impl Soul {
    pub fn empty(id: SoulID) -> Self {
        Self {
            id,
            desires: vec![],
        }
    }

    pub fn human_soul(id: SoulID, house: BuildingID, goria: &mut Egregoria) -> Option<Self> {
        let map = goria.read::<Map>();
        let work = map
            .random_building(BuildingKind::Workplace, &mut *goria.write::<RandProvider>())?
            .id;
        drop(map);

        goria.write::<BuildingInfos>().add_owner(house, id);

        let body = spawn_pedestrian(goria, house);

        let offset = goria.write::<RandProvider>().random::<f32>() * 0.5;

        Some(Self {
            id,
            desires: vec![
                Box::new(Work::new(body, work, offset)),
                Box::new(Home::new(body, house, offset)),
            ],
        })
    }

    pub fn decision(&mut self, goria: &Egregoria) -> Action {
        self.desires
            .iter()
            .max_by_key(|d| OrderedFloat(d.score(goria)))
            .map(move |d| d.apply(goria))
            .unwrap_or_default()
    }
}
