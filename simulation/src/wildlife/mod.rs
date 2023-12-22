use crate::wildlife::bird::spawn_bird;
use crate::Simulation;
use common::rand::{rand, rand2};
use geom::Vec3;

pub mod bird;

/// HACK (for now): add no more than 50 birds to random locations
pub(crate) fn add_birds_randomly(sim: &mut Simulation) {
    profiling::scope!("wildlife::add_birds_randomly");

    let num_birds = sim.world().birds.len();
    if num_birds > 50 {
        return;
    }

    let home_pos = Vec3 {
        x: 5000.0 + 1000.0 * rand(num_birds as f32),
        y: 5000.0 + 1000.0 * rand2(num_birds as f32, num_birds as f32),
        z: 0.0,
    };

    spawn_bird(sim, home_pos);
}
