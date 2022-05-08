use crate::map_dynamic::Itinerary;
use crate::pedestrians::Pedestrian;
use crate::physics::Kinematics;
use crate::utils::time::GameTime;
use geom::{angle_lerpxy, Transform, Vec3};
use hecs::World;
use rayon::iter::ParallelBridge;
use rayon::prelude::*;
use resources::Resources;

#[profiling::function]
pub fn pedestrian_decision_system(world: &mut World, resources: &mut Resources) {
    let ra = &*resources.get().unwrap();
    world
        .query::<(
            &mut Itinerary,
            &mut Transform,
            &mut Kinematics,
            &mut Pedestrian,
        )>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|batch| batch.for_each(|(_, (a, b, c, d))| pedestrian_decision(ra, a, b, c, d)))
}

pub fn pedestrian_decision(
    time: &GameTime,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Kinematics,
    pedestrian: &mut Pedestrian,
) {
    let (desired_v, desired_dir) = calc_decision(pedestrian, trans, it);

    pedestrian.walk_anim += 7.0 * kin.speed * time.delta / pedestrian.walking_speed;
    physics(kin, trans, time, desired_v, desired_dir);
}

const PEDESTRIAN_ACC: f32 = 1.5;

pub fn physics(
    kin: &mut Kinematics,
    trans: &mut Transform,
    time: &GameTime,
    desired_velocity: f32,
    desired_dir: Vec3,
) {
    let diff = desired_velocity - kin.speed;
    let mag = diff.min(time.delta * PEDESTRIAN_ACC);
    if mag > 0.0 {
        kin.speed += mag;
    }
    const ANG_VEL: f32 = 1.0;
    trans.dir = angle_lerpxy(trans.dir, desired_dir, ANG_VEL * time.delta);
}

pub fn calc_decision(
    pedestrian: &mut Pedestrian,
    trans: &Transform,
    it: &Itinerary,
) -> (f32, Vec3) {
    let objective = match it.get_point() {
        Some(x) => x,
        None => return (0.0, trans.dir),
    };

    let position = trans.position;

    let delta_pos: Vec3 = objective - position;
    let dir_to_pos = match delta_pos.try_normalize() {
        Some(x) => x,
        None => return (0.0, trans.dir),
    };

    let desired_dir = dir_to_pos.normalize();
    (pedestrian.walking_speed, desired_dir)
}
