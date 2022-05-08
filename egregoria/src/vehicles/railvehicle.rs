use crate::{GameTime, Itinerary, Kinematics};
use geom::{Color, Transform};
use hecs::{Entity, World};
use map_model::Map;
use rayon::iter::{ParallelBridge, ParallelIterator};
use resources::Resources;

pub struct RailVehicle {
    pub max_speed: f32,
    pub deceleration_force: f32,
    pub tint: Color,
}

pub struct RailWagon {
    pub locomotive: Entity,
}

#[profiling::function]
pub fn railvehicle_decision_system(world: &mut World, resources: &mut Resources) {
    let ra = &*resources.get().unwrap();
    let rb = &*resources.get().unwrap();
    world
        .query::<(
            &mut Itinerary,
            &mut Transform,
            &mut Kinematics,
            &mut RailVehicle,
        )>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|batch| {
            batch.for_each(|(ent, (a, b, c, d))| {
                railvehicle_decision(ra, rb, ent, a, b, c, d);
            })
        })
}

pub fn railvehicle_decision(
    map: &Map,
    time: &GameTime,
    me: Entity,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Kinematics,
    railvehicle: &mut RailVehicle,
) {
    let time_to_stop = kin.speed * kin.speed / (2.0 * railvehicle.deceleration_force);
}
