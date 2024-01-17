#![allow(dead_code)]
#![cfg(test)]

use crate::map::{BuildingID, LanePatternBuilder, ProjectFilter};
use crate::map_dynamic::BuildingInfos;
use crate::utils::scheduler::SeqSchedule;
use crate::world_command::{WorldCommand, WorldCommands};
use crate::{Simulation, SimulationOptions};
use common::logger::MyLog;
use common::saveload::Encoder;
use geom::{Vec2, Vec3};

mod test_iso;
mod vehicles;

pub(crate) struct TestCtx {
    pub g: Simulation,
    sched: SeqSchedule,
}

impl TestCtx {
    pub(crate) fn new() -> Self {
        MyLog::init();
        crate::init::init();

        let g = Simulation::new_with_options(SimulationOptions {
            terrain_size: 1,
            save_replay: false,
        });
        let sched = Simulation::schedule();

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

        let serialized = common::saveload::Bincode::encode(&self.g).unwrap();
        let deserialized: Simulation = common::saveload::Bincode::decode(&serialized).unwrap();

        let testhashes = self.g.hashes();
        for (key, hash) in deserialized.hashes().iter() {
            assert_eq!(
                testhashes.get(key),
                Some(hash),
                "key: {:?} at tick {}",
                key,
                self.g.get_tick(),
            );
        }
    }
}
