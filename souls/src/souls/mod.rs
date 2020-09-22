use crate::desire::Desire;
use crate::souls::human::Human;
use egregoria::api::Action;
use egregoria::engine_interaction::RenderStats;
use egregoria::map_dynamic::BuildingInfos;
use egregoria::{Egregoria, SoulID};
use map_model::{BuildingKind, Map};
use ordered_float::OrderedFloat;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use std::time::Instant;

mod human;

pub struct Soul<T> {
    pub id: SoulID,
    desires: Vec<Box<dyn Desire<T>>>,
    extra: T,
}

impl<T> Soul<T> {
    pub fn decision(&mut self, goria: &Egregoria) -> Action {
        let extra = &mut self.extra;
        self.desires
            .iter_mut()
            .max_by_key(|d| OrderedFloat(d.score(goria, extra)))
            .map(move |d| d.apply(goria, extra))
            .unwrap_or_default()
    }
}

#[derive(Default)]
pub struct Souls {
    human_souls: Vec<Soul<Human>>,
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

        let mut n_souls_added = 0;

        for house in empty_buildings {
            let id = SoulID(self.human_souls.len() as u64);

            if let Some(soul) = Human::soul(id, house, goria) {
                self.human_souls.push(soul);
                n_souls_added += 1;
                if n_souls_added > 100 {
                    break;
                }
            }
        }

        if n_souls_added > 0 {
            log::info!("{} souls added", n_souls_added);
        }
    }

    pub fn update(&mut self, goria: &mut Egregoria) {
        let refgoria = &*goria;
        let t = Instant::now();
        let actions: Vec<Action> = self
            .human_souls
            .par_iter_mut()
            .map(move |x: &mut Soul<Human>| x.decision(refgoria))
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
