use crate::utils::rand_provider::RandProvider;
use crate::wildlife::bird::{
    get_random_bird_pos, get_random_pos_from_center, spawn_bird, NUM_FLOCKS,
};
use crate::{BirdID, Flock, Simulation};

pub mod bird;

const BIRDS_PER_FLOCK: u32 = 50;
const DIST_SPREAD: f32 = 5.0; // how large of a radius to initially spawn birds

/// HACK (for now): spawns birds in random clusters around the map
pub(crate) fn add_flocks_randomly(sim: &mut Simulation) {
    profiling::scope!("wildlife::add_flocks_randomly");

    let num_flocks = sim.world().flocks.len();
    if num_flocks >= NUM_FLOCKS as usize {
        return;
    }

    let mut rng = RandProvider::new(num_flocks as u64);

    let aabb = sim.map().terrain.bounds();
    let center_pos = get_random_bird_pos(aabb, rng.next_f32(), rng.next_f32(), rng.next_f32());

    let mut ids: Vec<BirdID> = Vec::new();

    for _ in 0..BIRDS_PER_FLOCK {
        let bird_pos = get_random_pos_from_center(
            center_pos,
            DIST_SPREAD,
            rng.next_f32(),
            rng.next_f32(),
            rng.next_f32(),
        );
        match spawn_bird(sim, bird_pos) {
            Some(id) => ids.push(id),
            None => (),
        }
    }

    sim.world.insert(Flock { bird_ids: ids });
}
