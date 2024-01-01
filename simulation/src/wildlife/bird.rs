use crate::map::Map;
use crate::physics::Speed;
use crate::utils::resources::Resources;
use crate::utils::time::GameTime;
use crate::Simulation;
use crate::World;
use crate::{BirdEnt, BirdID};
use geom::angle_lerpxy;
use geom::AABB;
use geom::{Transform, Vec3};

pub fn spawn_bird(sim: &mut Simulation, spawn_pos: Vec3) -> Option<BirdID> {
    profiling::scope!("spawn_bird");

    log::info!("added bird at {}", spawn_pos);

    let id = sim.world.insert(BirdEnt {
        trans: Transform::new(spawn_pos),
        speed: Speed::default(),
    });

    Some(id)
}

pub fn bird_decision_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("wildlife::animal_decision_system");
    let ra = &*resources.read::<GameTime>();
    let map = &*resources.read::<Map>();

    let aabb = map.environment.bounds();

    world.flocks.values().for_each(|flock| {
        let flock_physics: Vec<(Transform, Speed)> = flock
            .bird_ids
            .iter()
            .map(|bird_id| match world.birds.get_mut(*bird_id) {
                Some(bird_ent) => (bird_ent.trans, bird_ent.speed.clone()),
                None => unreachable!(),
            })
            .collect();

        let flock_center = center(&flock_physics);
        let flock_avg_v = average_velocity(&flock_physics);

        flock
            .bird_ids
            .iter()
            .for_each(|bird_id| match world.birds.get_mut(*bird_id) {
                Some(bird_ent) => bird_decision(
                    ra,
                    &mut bird_ent.trans,
                    &mut bird_ent.speed,
                    flock_center,
                    flock_avg_v,
                    aabb,
                    &flock_physics,
                ),
                None => unreachable!(),
            })
    });
}

/// Update the speed, position, and direction of a bird
pub fn bird_decision(
    time: &GameTime,
    trans: &mut Transform,
    kin: &mut Speed,
    flock_center: Vec3,
    flock_avg_v: Vec3,
    aabb: AABB,
    flock_physics: &Vec<(Transform, Speed)>,
) {
    // the initial velocity of the bird
    let mut dv = trans.dir * kin.0;

    // fly towards the average position of all other birds
    const CENTERING_FACTOR: f32 = 0.01;
    let num_birds = flock_physics.len() as f32;
    let perceived_center = (flock_center * num_birds - trans.position) / (num_birds - 1.0);
    dv += (perceived_center - trans.position) * CENTERING_FACTOR;

    // match the flock's average velocity
    const MATCHING_FACTOR: f32 = 0.01;
    dv += (flock_avg_v - dv) * MATCHING_FACTOR;

    // avoid nearby birds
    const AVOID_FACTOR: f32 = 0.01;
    dv += separation_adjustment(trans, flock_physics) * AVOID_FACTOR;

    // avoid map boundaries
    dv += bounds_adjustment(trans, aabb);

    // cap the speed of the bird
    const SPEED_LIMIT: f32 = 10.0;
    if dv.mag() > SPEED_LIMIT {
        dv = dv.normalize_to(SPEED_LIMIT);
    }

    // update the bird's speed, position, and direction
    const ANG_VEL: f32 = 1.0;
    trans.dir = angle_lerpxy(trans.dir, dv, ANG_VEL * time.realdelta).normalize();
    kin.0 = dv.mag();
    trans.position += dv * time.realdelta;
}

/// Calculate the center of the flock (the average position of the flock)
fn center(flock_physics: &Vec<(Transform, Speed)>) -> Vec3 {
    flock_physics
        .iter()
        .map(|(t, _)| t.position)
        // TODO: use .sum() ?
        .reduce(|a, b| a + b).unwrap()
        / flock_physics.len() as f32
}

/// Calculate the average velocity of the flock
fn average_velocity(flock_physics: &Vec<(Transform, Speed)>) -> Vec3 {
    flock_physics
        .iter()
        .map(|(t, s)| t.dir.normalize() * s.0)
        // TODO: use .sum() ?
        .reduce(|a, b| a + b).unwrap()
        / flock_physics.len() as f32
}

/// Get an adjustment vector to move the bird away from other birds
fn separation_adjustment(trans: &Transform, flock_physics: &Vec<(Transform, Speed)>) -> Vec3 {
    const MIN_DISTANCE: f32 = 5.0;
    flock_physics
        .iter()
        .filter(|(other, _)| other.position.distance(trans.position) < MIN_DISTANCE)
        .map(|(other, _)| trans.position - other.position)
        // TODO: use .sum() ?
        .reduce(|a, b| a + b)
        .unwrap()
}

/// Get an adjustment vector to move the bird away from the map bounds
fn bounds_adjustment(trans: &Transform, aabb: AABB) -> Vec3 {
    const MARGIN: f32 = 2.0;
    const MAX_Z: f32 = 200.0;
    const TURN_AMOUNT: f32 = 1.0;
    let mut v = Vec3::new(0.0, 0.0, 0.0);
    // TODO: the ground might not be at 0
    if trans.position.z < MARGIN {
        v.z += TURN_AMOUNT;
    }
    if trans.position.z > MAX_Z - MARGIN {
        v.z -= TURN_AMOUNT;
    }
    if trans.position.x < aabb.ll.x + MARGIN {
        v.x += TURN_AMOUNT;
    }
    if trans.position.x > aabb.ur.x - MARGIN {
        v.x -= TURN_AMOUNT;
    }
    if trans.position.y < aabb.ll.y + MARGIN {
        v.y += TURN_AMOUNT;
    }
    if trans.position.y > aabb.ur.y - MARGIN {
        v.y -= TURN_AMOUNT;
    }
    v
}
