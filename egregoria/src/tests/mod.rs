#![allow(dead_code)]
#![cfg(test)]

use crate::engine_interaction::{WorldCommand, WorldCommands};
use crate::map::{BuildingID, LanePatternBuilder, ProjectFilter};
use crate::map_dynamic::BuildingInfos;
use crate::utils::scheduler::SeqSchedule;
use crate::{Egregoria, EgregoriaOptions};
use common::logger::MyLog;
use geom::{Vec2, Vec3};

mod vehicles;

pub(crate) struct TestCtx {
    pub g: Egregoria,
    sched: SeqSchedule,
}

impl TestCtx {
    pub(crate) fn new() -> Self {
        MyLog::init();
        crate::init::init();

        let g = Egregoria::new_with_options(EgregoriaOptions {
            terrain_size: 1,
            save_replay: false,
            ..Default::default()
        });
        let sched = Egregoria::schedule();

        Self { g, sched }
    }

    pub(crate) fn build_roads(&self, v: &[Vec3]) {
        let mut m = self.g.map_mut();
        for w in v.windows(2) {
            let a = m.project(w[0], 0.0, ProjectFilter::ALL);
            let b = m.project(w[1], 0.0, ProjectFilter::ALL);
            m.make_connection(a, b, None, &LanePatternBuilder::default().build());
        }
    }

    pub(crate) fn build_house_near(&self, p: Vec2) -> BuildingID {
        let lot = self
            .g
            .map()
            .lots()
            .values()
            .min_by_key(|lot| lot.shape.center().distance2(p) as i32)
            .unwrap()
            .id;

        let b = self.g.map_mut().build_house(lot).unwrap();
        self.g.write::<BuildingInfos>().insert(b);
        b
    }

    pub(crate) fn apply(&mut self, commands: &[WorldCommand]) {
        for c in commands {
            c.apply(&mut self.g);
        }
    }

    pub(crate) fn tick(&mut self) {
        self.g
            .tick(&mut self.sched, WorldCommands::default().as_ref());
    }
}
