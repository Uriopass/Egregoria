use geom::{Vec3, AABB};

use crate::utils::rand_provider::RandProvider;
use crate::wildlife::bird::spawn_bird;
use crate::{BirdID, Flock, Simulation};

pub mod bird;

const MIN_SPAWN_HEIGHT: f32 = 2.0;
const SPAWN_HEIGHT_RANGE: f32 = 10.0;

/// Get a random position within the bounding box
pub fn get_random_spawn_pos(aabb: AABB, r1: f32, r2: f32, r3: f32) -> Vec3 {
    let AABB { ll, ur } = aabb;
    Vec3 {
        x: ll.x + (ur.x - ll.x) * r1,
        y: ll.y + (ur.y - ll.y) * r2,
        z: MIN_SPAWN_HEIGHT + SPAWN_HEIGHT_RANGE * r3,
    }
}

/// Get a random position within the ball with the given center and radius
pub fn get_random_pos_from_center(center: Vec3, radius: f32, r1: f32, r2: f32, r3: f32) -> Vec3 {
    Vec3 {
        x: center.x + radius * (r1 - 0.5),
        y: center.y + radius * (r2 - 0.5),
        z: MIN_SPAWN_HEIGHT + SPAWN_HEIGHT_RANGE * r3,
    }
}

const NUM_FLOCKS: u32 = 20;
const BIRDS_PER_FLOCK: u32 = 50;
const SPAWN_RANGE: f32 = 5.0; // how spread out birds in the flock should be initially

/// HACK (for now): spawns birds in random clusters around the map
pub(crate) fn add_flocks_randomly(sim: &mut Simulation) {
    profiling::scope!("wildlife::add_flocks_randomly");

    let num_flocks = sim.world().flocks.len();
    if num_flocks >= NUM_FLOCKS as usize {
        return;
    }

    let mut rng = RandProvider::new(num_flocks as u64);

    let aabb = sim.map().environment.bounds();
    let center_pos = get_random_spawn_pos(aabb, rng.next_f32(), rng.next_f32(), rng.next_f32());

    let mut ids: Vec<BirdID> = Vec::new();

    for _ in 0..BIRDS_PER_FLOCK {
        let bird_pos = get_random_pos_from_center(
            center_pos,
            SPAWN_RANGE,
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
