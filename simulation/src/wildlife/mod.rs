use crate::utils::rand_provider::RandProvider;
use crate::wildlife::bird::{
    get_random_bird_pos, get_random_pos_from_center, spawn_bird, NUM_FLOCKS,
};
use crate::Simulation;
use common::rand::{rand, rand2, rand3};

pub mod bird;

const NUM_BIRDS: u32 = 1000;

/// HACK (for now): spawns birds in random clusters around the map
pub(crate) fn add_birds_randomly(sim: &mut Simulation) {
    profiling::scope!("wildlife::add_birds_randomly");

    let num_birds = sim.world().birds.len();
    if num_birds > NUM_BIRDS as usize {
        return;
    }

    let aabb = sim.map().terrain.bounds();
    let seed = (num_birds as u32 / (NUM_BIRDS / NUM_FLOCKS)) as f32;
    let cluster_pos =
        get_random_bird_pos(aabb, rand(seed), rand2(seed, seed), rand3(seed, seed, seed));

    let mut rng = RandProvider::new(num_birds as u64);

    let home_pos = get_random_pos_from_center(
        cluster_pos,
        5.0,
        rng.next_f32(),
        rng.next_f32(),
        rng.next_f32(),
    );
    spawn_bird(sim, home_pos);
}
