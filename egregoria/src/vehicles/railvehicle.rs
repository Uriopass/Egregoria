use crate::map_dynamic::ItineraryKind;
use crate::{
    Egregoria, GameTime, Itinerary, ItineraryFollower, ItineraryLeader, Kinematics, Selectable,
};
use geom::{PolyLine3, Polyline3Queue, Transform, Vec3};
use hecs::{Entity, World};
use imgui_inspect_derive::*;
use map_model::{LaneID, Map, PathKind};
use rayon::iter::{ParallelBridge, ParallelIterator};
use resources::Resources;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Inspect)]
pub struct Locomotive {
    /// m/s
    pub max_speed: f32,
    /// m.s^2
    pub acc_force: f32,
    /// m.s^2
    pub dec_force: f32,
}

#[derive(Serialize, Deserialize)]
pub struct RandomLocomotive;

#[derive(Serialize, Deserialize)]
pub struct RailWagon;

const WAGON_INTERLENGTH: f32 = 16.75;

pub fn wagons_dists_to_loco(n_wagons: u32) -> impl DoubleEndedIterator<Item = f32> {
    (1..n_wagons + 1).map(|x| 1.0 + x as f32 * 16.75)
}

pub fn wagons_positions(
    points: &PolyLine3,
    dist: f32,
    n_wagons: u32,
) -> impl Iterator<Item = (Vec3, Vec3)> + '_ {
    let positions = std::iter::once(0.0)
        .chain(wagons_dists_to_loco(n_wagons))
        .rev()
        .filter_map(move |wdist| {
            let pos = dist - wdist;
            if pos >= 0.0 {
                Some(pos)
            } else {
                None
            }
        });

    points.points_dirs_along(positions)
}

pub fn spawn_train(
    goria: &mut Egregoria,
    dist: f32,
    n_wagons: u32,
    lane: LaneID,
) -> Option<Entity> {
    let (world, res) = goria.world_res();

    let map = res.get::<Map>().ok()?;
    let lane = map.lanes().get(lane)?;

    let (locopos, locodir) = lane.points.point_dir_along(dist);

    let (_, segment) = lane.points.project_segment(locopos);

    let mut points = lane
        .points
        .iter()
        .take(segment)
        .copied()
        .collect::<Vec<_>>();
    points.reverse();

    let leader = ItineraryLeader {
        past: Polyline3Queue::new(
            points.into_iter(),
            locopos,
            10.0 + n_wagons as f32 * WAGON_INTERLENGTH,
        ),
    };

    let loco = world.spawn((
        Transform::new_dir(locopos, locodir),
        Kinematics::default(),
        Selectable::new(10.0),
        Locomotive {
            max_speed: 50.0,
            acc_force: 1.0,
            dec_force: 2.5,
        },
        RandomLocomotive,
        Itinerary::NONE,
    ));

    for mut follower in leader.past.mk_followers(wagons_dists_to_loco(n_wagons)) {
        let (pos, dir) = follower.update(&leader.past);
        world.spawn((
            Transform::new_dir(pos, dir),
            Kinematics::default(),
            Selectable::new(10.0),
            RailWagon,
            ItineraryFollower {
                leader: loco,
                follower,
            },
        ));
    }

    world.insert_one(loco, leader).unwrap();

    Some(loco)
}

#[profiling::function]
pub fn locomotive_random_movement_system(world: &mut World, resources: &mut Resources) {
    let map = &*resources.get::<Map>().unwrap();
    let time = &*resources.get::<GameTime>().unwrap();
    world
        .query::<(
            &mut Itinerary,
            &mut Transform,
            &Locomotive,
            &RandomLocomotive,
        )>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|batch| {
            batch.for_each(|(_, (itin, trans, _, _))| {
                let mut reroute = false;
                if let Some(t) = itin.get_terminal() {
                    if t.is_close(trans.position, 1.0) {
                        reroute = true;
                    }
                }
                if itin.is_none() || itin.is_wait_for_reroute().is_some() {
                    reroute = true;
                }

                if reroute {
                    if let Some(r) = map.lanes().values().nth(
                        (map.lanes().len() as f32
                            * common::rand::rand3(
                                trans.position.x,
                                trans.position.y,
                                time.seconds as f32,
                            )) as usize,
                    ) {
                        if r.kind.is_rail() {
                            *itin = Itinerary::route(
                                trans.position,
                                r.points.last(),
                                map,
                                PathKind::Rail,
                            )
                            .unwrap_or(Itinerary::NONE);
                        }
                    }
                }
            })
        })
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
