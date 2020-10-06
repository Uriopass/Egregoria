use crate::desire::Desires;
use crate::souls::human::{Human, HumanSoul};
use crate::souls::supermarket::SupermarketSoul;
use crate::supermarket::Supermarket;
use crate::DebugSoul;
use common::inspect::InspectedEntity;
use common::GameTime;
use egregoria::api::Action;
use egregoria::engine_interaction::{History, RenderStats};
use egregoria::map_dynamic::BuildingInfos;
use egregoria::pedestrians::{Pedestrian, PedestrianID};
use egregoria::{Egregoria, SoulID};
use map_model::{BuildingKind, Map};
use rayon::iter::ParallelIterator;
use rayon::iter::{IntoParallelRefMutIterator, ParallelExtend};
use std::collections::HashMap;
use std::time::Instant;

pub mod human;
pub mod supermarket;

pub struct Soul<T, D: Desires<T>> {
    desires: D,
    extra: T,
}

#[derive(Default)]
pub struct Souls {
    pub growing: usize,
    human_souls: HashMap<SoulID, HumanSoul>,
    supermarket_souls: HashMap<SoulID, SupermarketSoul>,
    body_map: HashMap<PedestrianID, SoulID>,
}

impl Souls {
    pub fn fresh_id(&mut self) -> SoulID {
        let id = SoulID(self.growing);
        self.growing += 1;
        id
    }

    pub fn add_souls_to_empty_buildings(&mut self, goria: &mut Egregoria) {
        let map = goria.read::<Map>();
        let infos = goria.read::<BuildingInfos>();
        let mut empty_buildings = vec![];
        for (id, building) in map.buildings() {
            if !matches!(
                building.kind,
                BuildingKind::House | BuildingKind::Supermarket
            ) {
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
            let id = self.fresh_id();

            match kind {
                BuildingKind::House => {
                    if let Some(soul) = Human::soul(goria, id, build_id) {
                        self.body_map.insert(soul.extra.router.body, id);
                        self.human_souls.insert(id, soul);

                        n_souls_added += 1;
                    }
                }
                BuildingKind::Supermarket => {
                    let soul = Supermarket::soul(goria, id, build_id);
                    self.supermarket_souls.insert(id, soul);
                    n_souls_added += 1;
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

    pub fn update(&mut self, goria: &mut Egregoria) {
        goria.set_read_only(true);
        let refgoria = &*goria;
        let t = Instant::now();
        let mut actions: Vec<Action> = vec![];

        actions.par_extend(
            self.human_souls
                .par_iter_mut()
                .map(move |(_, x): (_, &mut HumanSoul)| x.desires.decision(&mut x.extra, refgoria)),
        );

        #[allow(clippy::unit_arg)] // Fixme: remove this when supermarket's soul gets a desire
        actions.par_extend(self.supermarket_souls.par_iter_mut().map(
            move |(_, x): (_, &mut SupermarketSoul)| x.desires.decision(&mut x.extra, refgoria),
        ));

        goria.set_read_only(false);

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
            if goria.comp::<Pedestrian>(x).is_some() && goria.read::<GameTime>().tick(1) {
                let soul_id = self
                    .body_map
                    .get(&PedestrianID(x))
                    .expect("soul with pedestrian wasn't added to body_map");
                let soul = &self.human_souls[&soul_id];

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
