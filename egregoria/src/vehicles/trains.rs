use crate::map_dynamic::ItineraryKind;
use crate::{
    Egregoria, GameTime, Itinerary, ItineraryFollower, ItineraryLeader, Kinematics, Selectable,
};
use geom::{PolyLine3, Polyline3Queue, Transform, Vec3};
use hecs::{Entity, View, World};
use imgui_inspect_derive::*;
use map_model::{IntersectionID, LaneID, Map, PathKind, TraverseKind};
use rayon::iter::{ParallelBridge, ParallelIterator};
use resources::Resources;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

#[derive(Default, Serialize, Deserialize)]
pub struct TrainReservations {
    pub reservations: BTreeMap<IntersectionID, Entity>,
    pub localisations: BTreeMap<LaneID, BTreeMap<Entity, f32>>,
}

#[derive(Serialize, Deserialize, Inspect)]
pub struct Locomotive {
    /// m/s
    pub max_speed: f32,
    /// m.s^2
    pub acc_force: f32,
    /// m.s^2
    pub dec_force: f32,
    /// m
    pub length: f32,
}

#[derive(Serialize, Deserialize, Inspect)]
pub struct LocomotiveReservation {
    pub cur_travers_dist: f32,
    past_travers: BTreeMap<TraverseKind, f32>,
    upcoming_inters: Vec<IntersectionID>,
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

    let train_length = 1.0 + (n_wagons + 1) as f32 * WAGON_INTERLENGTH;

    let leader = ItineraryLeader {
        past: Polyline3Queue::new(points.into_iter(), locopos, train_length + 20.0),
    };

    let loco = world.spawn((
        Transform::new_dir(locopos, locodir),
        Kinematics::default(),
        Selectable::new(10.0),
        Locomotive {
            max_speed: 50.0,
            acc_force: 1.0,
            dec_force: 2.5,
            length: train_length,
        },
        LocomotiveReservation {
            cur_travers_dist: dist,
            past_travers: BTreeMap::from([(
                TraverseKind::Lane(lane.id),
                dist - lane.points.length(),
            )]),
            upcoming_inters: Default::default(),
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

pub fn traverse_forward<'a>(
    map: &'a Map,
    itin: &'a Itinerary,
    dist: f32,
    mut acc: f32,
) -> impl Iterator<Item = (TraverseKind, f32)> + 'a {
    let mut it = None;
    if let ItineraryKind::Route(route, _) = itin.kind() {
        it = Some(route);
    }
    let lanes = map.lanes();
    let inters = map.intersections();
    it.into_iter()
        .flat_map(move |route| route.reversed_route.iter().rev())
        .filter_map(move |v| {
            let oldacc = acc;
            acc += v.kind.length(lanes, inters)?;
            Some((v.kind, oldacc))
        })
        .take_while(move |(_, acc)| *acc < dist)
}

#[profiling::function]
pub fn train_reservations_update(world: &mut World, resources: &mut Resources) {
    let map = &*resources.get::<Map>().unwrap();
    let reservations = &mut *resources.get_mut::<TrainReservations>().unwrap();
    let lanes = map.lanes();
    let inters = map.intersections();
    world
        .query_mut::<(
            &Itinerary,
            &Locomotive,
            &mut LocomotiveReservation,
            &Kinematics,
        )>()
        .into_iter()
        .for_each(move |(me, (itin, loco, locores, kin))| {
            // Remember when we've been
            if let Some(travers) = itin.get_travers() {
                match locores.past_travers.entry(travers.kind) {
                    Entry::Vacant(v) => {
                        v.insert(-travers.kind.length(lanes, inters).unwrap_or(0.0));
                        locores.cur_travers_dist = 0.0;
                    }
                    Entry::Occupied(_) => {}
                };

                // Handle upcoming intersections
                // Start by cleaning them then re-reserve them (so that in event of weirdness, they stay correct)
                for v in locores.upcoming_inters.drain(..) {
                    reservations.reservations.remove(&v);
                }

                let dist_to_next =
                    travers.kind.length(lanes, inters).unwrap_or(0.0) - locores.cur_travers_dist;

                // Then look ahead stop_dist to reserve all intersections
                let stop_dist = kin.speed * kin.speed / (2.0 * loco.dec_force);
                for (v, _) in traverse_forward(map, itin, stop_dist + 15.0, dist_to_next) {
                    if let TraverseKind::Turn(id) = v {
                        if inters
                            .get(id.parent)
                            .map(|i| i.roads.len() <= 2)
                            .unwrap_or(true)
                        {
                            continue;
                        }

                        reservations
                            .reservations
                            .entry(id.parent)
                            .or_insert_with(|| {
                                locores.upcoming_inters.push(id.parent);
                                me
                            });
                    }
                }
            }

            // Clean past_things and unreserve them
            let length = loco.length;
            locores.past_travers.retain(|&id, dist| {
                match id {
                    TraverseKind::Lane(id) => {
                        reservations
                            .localisations
                            .entry(id)
                            .or_default()
                            .insert(me, *dist);
                        if *dist >= length {
                            unwrap_ret!(reservations.localisations.get_mut(&id), false).remove(&me);
                            return false;
                        }
                    }
                    TraverseKind::Turn(id) => {
                        reservations.reservations.entry(id.parent).or_insert(me);
                        if *dist >= length {
                            reservations.reservations.remove(&id.parent);
                            return false;
                        }
                    }
                }

                true
            });
        });
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
                            let segments: Vec<_> = r.points.segments().collect();

                            *itin = Itinerary::route(
                                trans.position,
                                segments[segments.len() / 2].middle(),
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
    let map = &*resources.get().unwrap();
    let time = &*resources.get().unwrap();
    let reservs = &*resources.get().unwrap();

    let mut locoqry = world.query::<&Locomotive>();
    let locoview = locoqry.view();

    world
        .query::<(
            &mut Itinerary,
            &mut Transform,
            &mut Kinematics,
            &Locomotive,
            &mut LocomotiveReservation,
        )>()
        .iter()
        .for_each(move |(ent, (it, trans, kin, loco, locores))| {
            locomotive_decision(
                map, time, reservs, &locoview, ent, it, trans, kin, loco, locores,
            );
        })
}

pub fn locomotive_decision(
    map: &Map,
    time: &GameTime,
    reservs: &TrainReservations,
    locoview: &View<&Locomotive>,
    me: Entity,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Kinematics,
    loco: &Locomotive,
    locores: &mut LocomotiveReservation,
) {
    let desired_speed =
        locomotive_desired_speed(me, map, reservs, locoview, trans, kin, it, loco, locores);
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
    for v in locores.past_travers.values_mut() {
        *v += kin.speed * time.delta;
    }
    locores.cur_travers_dist += kin.speed * time.delta;
}

pub fn locomotive_desired_speed(
    me: Entity,
    map: &Map,
    reservs: &TrainReservations,
    locoview: &View<&Locomotive>,
    trans: &Transform,
    kin: &Kinematics,
    it: &Itinerary,
    loco: &Locomotive,
    locores: &LocomotiveReservation,
) -> f32 {
    if matches!(it.kind(), ItineraryKind::None | ItineraryKind::WaitUntil(_)) {
        return 0.0;
    }

    let stop_dist = kin.speed * kin.speed / (2.0 * loco.dec_force);

    let mut lastid = None;
    let mydist = locores.cur_travers_dist;
    if let Some(travers) = it.get_travers() {
        let lanes = map.lanes();

        let dist_to_next = travers
            .kind
            .length(lanes, map.intersections())
            .unwrap_or(0.0)
            - mydist;

        for (id, acc) in std::iter::once((travers.kind, -mydist)).chain(traverse_forward(
            map,
            it,
            stop_dist + 15.0,
            dist_to_next,
        )) {
            match id {
                TraverseKind::Lane(id) => {
                    if let Some(locs) = reservs.localisations.get(&id) {
                        for (&train, &otherdist) in locs {
                            if train == me {
                                continue;
                            }
                            if let Some(otherloco) = locoview.get(train) {
                                let dist_to_other = acc
                                    + otherdist
                                    + lanes.get(id).map(|v| v.points.length()).unwrap_or(0.0);
                                if dist_to_other > 0.0
                                    && dist_to_other < otherloco.length + stop_dist + 10.0
                                {
                                    return 0.0;
                                }
                            }
                        }
                    }
                }
                TraverseKind::Turn(id) => {
                    if let Some(inter) = map.intersections().get(id.parent) {
                        if inter.roads.len() > 2 {
                            if let Some(reserved_by) = reservs.reservations.get(&id.parent) {
                                if *reserved_by != me {
                                    return 0.0;
                                }
                            }
                        }
                    }
                }
            }
            lastid = Some(id);
        }
    }

    let mut on_last_lane = false;
    if let ItineraryKind::Route(r, _) = it.kind() {
        if r.reversed_route.is_empty()
            || (lastid.is_some() && lastid == r.reversed_route.first().map(|x| x.kind))
        {
            on_last_lane = true;
        }
    }

    if matches!(it.kind(), ItineraryKind::Simple(_)) {
        on_last_lane = true
    }

    if on_last_lane {
        if let Some(howfar) = it.end_pos().map(|term| term.distance(trans.position)) {
            if howfar + 0.1 <= stop_dist {
                return 0.0;
            }
        }
    }

    loco.max_speed
}
