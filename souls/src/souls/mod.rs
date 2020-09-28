use crate::desire::{Desires, Routed};
use crate::souls::human::{Human, HumanSoul};
use crate::DebugSoul;
use common::inspect::InspectedEntity;
use egregoria::api::Action;
use egregoria::engine_interaction::{History, RenderStats, TimeInfo};
use egregoria::map_dynamic::BuildingInfos;
use egregoria::pedestrians::{Pedestrian, PedestrianID};
use egregoria::{Egregoria, SoulID};
use map_model::{BuildingKind, Map};
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use std::time::Instant;

mod human;

pub struct Soul<T, D: Desires<T>> {
    pub id: SoulID,
    desires: D,
    extra: T,
}

#[derive(Default)]
pub struct Souls {
    human_souls: Vec<HumanSoul>,
    body_map: HashMap<PedestrianID, SoulID>,
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
            let id = SoulID(self.human_souls.len());

            if let Some(mut soul) = Human::soul(id, house, goria) {
                self.body_map.insert(soul.extra.router_mut().body, id);
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
            .map(move |x: &mut HumanSoul| x.desires.decision(&mut x.extra, refgoria))
            .collect();

        goria
            .write::<RenderStats>()
            .souls_desires
            .add_value(t.elapsed().as_secs_f32());

        let t = Instant::now();
        for action in actions {
            let _ = action.apply(goria);
        }
        goria
            .write::<RenderStats>()
            .souls_apply
            .add_value(t.elapsed().as_secs_f32());

        goria.write_or_default::<DebugSoul>();

        if let Some(x) = goria.read::<InspectedEntity>().e {
            if goria.comp::<Pedestrian>(x).is_some() && goria.read::<TimeInfo>().tick(1) {
                let soul_id = self
                    .body_map
                    .get(&PedestrianID(x))
                    .expect("soul with pedestrian wasn't added to body_map");
                let soul = &self.human_souls[soul_id.0];

                let dbg: &mut DebugSoul = &mut *goria.write::<DebugSoul>();
                if dbg.cur_inspect.map(|p| p.0 != x).unwrap_or(true) {
                    dbg.scores.clear();
                }
                dbg.cur_inspect = Some(PedestrianID(x));

                for (i, (score, name)) in soul.desires.scores_names(goria, &soul.extra).enumerate()
                {
                    if i >= dbg.scores.len() {
                        dbg.scores.push((name, History::default()));
                    }
                    dbg.scores[i].1.add_value(score);
                }

                dbg.router = Some(soul.extra.router.clone());
            }
        } else {
            goria.write::<DebugSoul>().cur_inspect = None;
        }
    }
}
