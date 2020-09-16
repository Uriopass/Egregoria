use crate::desire::{Desire, Home, Work};
use egregoria::api::Action;
use egregoria::engine_interaction::RenderStats;
use egregoria::map_dynamic::BuildingInfos;
use egregoria::pedestrians::spawn_pedestrian;
use egregoria::utils::rand_provider::RandProvider;
use egregoria::{Egregoria, SoulID};
use map_model::{BuildingID, BuildingKind, Map};
use ordered_float::OrderedFloat;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::time::Instant;

mod desire;

#[derive(Default)]
pub struct Souls {
    souls: Vec<Soul>,
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
            let id = SoulID(self.souls.len() as u64);

            if let Some(soul) = Soul::human_soul(id, house, goria) {
                self.souls.push(soul);
            }
        }
    }

    pub fn update(&mut self, goria: &mut Egregoria) {
        let refgoria = PlsNoModify(&*goria);
        let t = Instant::now();
        let actions: Vec<_> = self
            .souls
            .par_iter()
            .map(|x| x.decision(refgoria.0))
            .collect();

        goria
            .write::<RenderStats>()
            .souls_desires
            .add_time(t.elapsed().as_secs_f32());

        let t = Instant::now();
        for action in actions {
            let _ = action.apply(goria);
        }
        goria
            .write::<RenderStats>()
            .souls_apply
            .add_time(t.elapsed().as_secs_f32());
    }
}

struct PlsNoModify<'a>(&'a Egregoria);

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

    pub fn decision(&self, goria: &Egregoria) -> Action {
        self.desires
            .iter()
            .max_by_key(|d| OrderedFloat(d.score(goria)))
            .map(move |d| d.apply(goria))
            .unwrap_or_default()
    }
}
