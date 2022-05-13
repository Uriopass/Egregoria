use crate::map_dynamic::ItineraryKind;
use crate::{GameTime, Itinerary, Kinematics};
use geom::Transform;
use hecs::{Entity, World};
use imgui_inspect_derive::*;
use map_model::Map;
use rayon::iter::{ParallelBridge, ParallelIterator};
use resources::Resources;

#[derive(Inspect)]
pub struct Locomotive {
    /// m/s
    pub max_speed: f32,
    /// m.s^2
    pub acc_force: f32,
    /// m.s^2
    pub dec_force: f32,
}

pub struct RailWagon;

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
    let desired_dir = it
        .get_point()
        .and_then(|x| {
            let d = x - trans.position;
            if d.magnitude2() < 0.5 {
                return None;
            }
            d.try_normalize()
        })
        .unwrap_or(trans.dir);
    trans.dir = desired_dir;

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
    let stop_dist = time_to_stop * kin.speed * 0.5;

    if let ItineraryKind::Route(r, _) = it.kind() {
        if r.reversed_route.is_empty() {
            if let Some(howfar) = it
                .local_path()
                .last()
                .map(|terminal| terminal.distance(trans.position))
            {
                if howfar + 0.1 <= stop_dist {
                    return 0.0;
                }
            }
        }
    }

    loco.max_speed
}
