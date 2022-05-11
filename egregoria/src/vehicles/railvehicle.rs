use crate::{GameTime, Itinerary, Kinematics};
use geom::{Transform, Vec3};
use hecs::{Entity, World};
use map_model::Map;
use rayon::iter::{ParallelBridge, ParallelIterator};
use resources::Resources;
use std::collections::VecDeque;

pub struct Locomotive {
    pub past: VecDeque<Vec3>,
    pub max_speed: f32,
    pub acc_force: f32,
    pub dec_force: f32,
}

pub struct RailWagon {
    pub locomotive: Entity,
}

#[profiling::function]
pub fn locomotive_system(world: &mut World, resources: &mut Resources) {
    let ra = &*resources.get().unwrap();
    let rb = &*resources.get().unwrap();
    world
        .query::<(
            &mut Itinerary,
            &mut Transform,
            &mut Kinematics,
            &mut Locomotive,
        )>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|batch| {
            batch.for_each(|(ent, (a, b, c, d))| {
                locomotive_decision(ra, rb, ent, a, b, c, d);
            })
        })
}

pub fn locomotive_decision(
    _map: &Map,
    time: &GameTime,
    _me: Entity,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Kinematics,
    loco: &mut Locomotive,
) {
    let desired_speed = locomotive_desired_speed(trans, kin, it, loco);
    trans.dir = it
        .get_point()
        .and_then(|x| (trans.position - x).try_normalize())
        .unwrap_or(trans.dir);

    kin.speed += (desired_speed - kin.speed)
        .clamp(-time.delta * loco.dec_force, time.delta * loco.acc_force);
}

pub fn locomotive_desired_speed(
    trans: &Transform,
    kin: &Kinematics,
    it: &Itinerary,
    loco: &Locomotive,
) -> f32 {
    let time_to_stop = kin.speed * kin.speed / (2.0 * loco.dec_force);

    let howfar = it
        .get_terminal()
        .map(|terminal| terminal.distance(trans.position))
        .unwrap_or(0.0);

    if howfar < time_to_stop {
        return 0.0;
    }
    loco.max_speed
}
