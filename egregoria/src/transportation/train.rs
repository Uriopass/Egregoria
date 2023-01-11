use crate::map::{IntersectionID, LaneID, Map, TraverseKind};
use crate::map_dynamic::{DispatchKind, ItineraryFollower2, ItineraryKind};
use crate::{Egregoria, GameTime, Itinerary, ItineraryLeader, Selectable, Speed};
use egui_inspect::Inspect;
use geom::{PolyLine3, Polyline3Queue, Transform, Vec3};
use hecs::{Entity, View, World};
use ordered_float::OrderedFloat;
use resources::Resources;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TrainID(pub Entity);

debug_inspect_impl!(TrainID);

#[derive(Default, Serialize, Deserialize)]
pub struct TrainReservations {
    pub reservations: BTreeMap<IntersectionID, Entity>,
    pub localisations: BTreeMap<TraverseKind, BTreeMap<Entity, f32>>,
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
    pub waited_for: f32,
    past_travers: BTreeMap<TraverseKind, f32>,
    upcoming_inters: Vec<IntersectionID>,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum RailWagonKind {
    Locomotive,
    Passenger,
    Fret,
}

#[derive(Serialize, Deserialize)]
pub struct RailWagon {
    pub kind: RailWagonKind,
}

const WAGON_INTERLENGTH: f32 = 16.75;

pub fn wagons_dists_to_loco(n_wagons: u32) -> impl DoubleEndedIterator<Item = f32> {
    (0..n_wagons + 1).map(|x| x as f32 * 16.75)
}

pub fn wagons_positions_for_render(
    points: &PolyLine3,
    dist: f32,
    n_wagons: u32,
) -> impl Iterator<Item = (Vec3, Vec3)> + '_ {
    let positions = std::iter::once(0.0)
        .chain(wagons_dists_to_loco(n_wagons).map(|x| x + WAGON_INTERLENGTH * 0.5))
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

pub fn train_length(n_wagons: u32) -> f32 {
    1.0 + (n_wagons + 1) as f32 * WAGON_INTERLENGTH
}

pub fn spawn_train(
    goria: &mut Egregoria,
    dist: f32,
    n_wagons: u32,
    lane: LaneID,
    kind: RailWagonKind,
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

    let trainlength = train_length(n_wagons);

    let leader = ItineraryLeader {
        past: Polyline3Queue::new(points.into_iter(), locopos, trainlength + 20.0),
    };

    let loco = world.spawn((
        Transform::new_dir(locopos, locodir),
        Speed::default(),
        Selectable::new(10.0),
        Locomotive {
            max_speed: 50.0,
            acc_force: 1.0,
            dec_force: 2.5,
            length: trainlength,
        },
        DispatchKind::FretTrain,
        LocomotiveReservation {
            cur_travers_dist: dist,
            waited_for: 0.0,
            past_travers: BTreeMap::from([(
                TraverseKind::Lane(lane.id),
                dist - lane.points.length(),
            )]),
            upcoming_inters: Default::default(),
        },
        Itinerary::NONE,
    ));
    let mut followers: Vec<_> = leader
        .past
        .mk_followers(
            wagons_dists_to_loco(n_wagons)
                .flat_map(|x| [x + WAGON_INTERLENGTH * 0.1, x + WAGON_INTERLENGTH * 0.9]),
        )
        .collect();
    for (i, follower) in followers.chunks_exact_mut(2).enumerate() {
        let (pos, dir) = follower[0].update(&leader.past);
        let (pos2, dir2) = follower[1].update(&leader.past);
        world.spawn((
            Transform::new_dir(pos * 0.5 + pos2 * 0.5, (0.5 * (dir + dir2)).normalize()),
            Speed::default(),
            Selectable::new(10.0),
            RailWagon {
                kind: if i == 0 {
                    RailWagonKind::Locomotive
                } else {
                    kind
                },
            },
            ItineraryFollower2 {
                leader: loco,
                head: follower[0],
                tail: follower[1],
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
    until_length: f32,
) -> impl Iterator<Item = (TraverseKind, f32, f32, f32)> + 'a {
    let mut it = None;
    if let ItineraryKind::Route(route, _) = itin.kind() {
        it = Some(route);
    }
    let lanes = map.lanes();
    let inters = map.intersections();
    let mut acc_inter = 0.0;
    it.into_iter()
        .flat_map(move |route| route.reversed_route.iter().rev())
        .filter_map(move |v| {
            let oldacc = acc;
            let l = v.kind.length(lanes, inters)?;

            match v.kind {
                TraverseKind::Turn(id) if inters.get(id.parent)?.roads.len() > 2 => acc_inter = 0.0,
                _ => acc_inter += l,
            }

            acc += l;
            Some((v.kind, oldacc, l, acc_inter))
        })
        .take_while(move |(_, acc, _, acc_l)| *acc < dist || *acc_l <= until_length)
}

#[profiling::function]
pub fn train_reservations_update(world: &mut World, resources: &mut Resources) {
    let map = &*resources.get::<Map>().unwrap();
    let reservations = &mut *resources.get_mut::<TrainReservations>().unwrap();
    let lanes = map.lanes();
    let inters = map.intersections();
    world
        .query_mut::<(&Itinerary, &Locomotive, &mut LocomotiveReservation, &Speed)>()
        .into_iter()
        .for_each(move |(me, (itin, loco, locores, kin))| {
            // Remember when we've been
            if let Some(travers) = itin.get_travers() {
                match locores.past_travers.entry(travers.kind) {
                    Entry::Vacant(v) => {
                        v.insert(10.0 - travers.kind.length(lanes, inters).unwrap_or(0.0));
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

                let mut want_to_reserve = vec![];
                let mut all_ok = true;
                // Then look ahead stop_dist to reserve all intersections
                let stop_dist = kin.speed * kin.speed / (2.0 * loco.dec_force);

                if let Some(v) = reservations.localisations.get(&travers.kind) {
                    if v.len() >= 2
                        && *v.values().max_by_key(|x| OrderedFloat(**x)).unwrap()
                            != v.get(&me).copied().unwrap_or(f32::NEG_INFINITY)
                    {
                        all_ok = false;
                    }
                }

                if all_ok {
                    for (id, _, _, _) in traverse_forward(
                        map,
                        itin,
                        stop_dist + 5.0,
                        dist_to_next,
                        loco.length + 25.0,
                    ) {
                        if let Some(v) = reservations.localisations.get(&id) {
                            if v.len() > 2 || (v.len() == 1 && v.get(&me).is_none()) {
                                all_ok = false;
                                break;
                            }
                        }
                        if let TraverseKind::Turn(id) = id {
                            if inters
                                .get(id.parent)
                                .map(|i| i.roads.len() <= 2)
                                .unwrap_or(true)
                            {
                                continue;
                            }

                            if reservations.reservations.get(&id.parent).is_some() {
                                all_ok = false;
                                break;
                            }
                            want_to_reserve.push(id.parent);
                        }
                    }

                    if all_ok {
                        for id in want_to_reserve {
                            reservations.reservations.insert(id, me);
                            locores.upcoming_inters.push(id);
                        }
                    }
                }
            }

            // Clean past_things and unreserve them
            let length = loco.length;
            locores.past_travers.retain(|&id, dist| {
                reservations
                    .localisations
                    .entry(id)
                    .or_default()
                    .insert(me, *dist);
                if *dist >= length {
                    if let TraverseKind::Turn(id) = id {
                        reservations.reservations.remove(&id.parent);
                    }
                    let l = unwrap_ret!(reservations.localisations.get_mut(&id), false);
                    l.remove(&me);
                    if l.is_empty() {
                        reservations.localisations.remove(&id);
                    }
                    return false;
                }
                if let TraverseKind::Turn(id) = id {
                    reservations.reservations.entry(id.parent).or_insert(me);
                }

                true
            });
        });
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
            &mut Speed,
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
    kin: &mut Speed,
    loco: &Locomotive,
    locores: &mut LocomotiveReservation,
) {
    let desired_speed =
        locomotive_desired_speed(me, map, reservs, locoview, trans, kin, it, loco, locores);
    let desired_dir = it
        .get_point()
        .and_then(|x| {
            let d = x - trans.position;
            if d.mag2() < 0.5 {
                return None;
            }
            d.try_normalize()
        })
        .unwrap_or(trans.dir);
    trans.dir = desired_dir;

    kin.speed += (desired_speed - kin.speed)
        .clamp(-time.delta * loco.dec_force, time.delta * loco.acc_force);
    if kin.speed <= 0.001 {
        locores.waited_for += time.delta;
    } else {
        locores.waited_for = 0.0;
    }
    for v in locores.past_travers.values_mut() {
        *v += kin.speed * time.delta;
        if locores.waited_for > 60.0 {
            *v += 0.1 * time.delta;
        }
    }
    locores.cur_travers_dist += kin.speed * time.delta;
}

pub fn locomotive_desired_speed(
    me: Entity,
    map: &Map,
    reservs: &TrainReservations,
    locoview: &View<&Locomotive>,
    trans: &Transform,
    kin: &Speed,
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

        let startl = travers
            .kind
            .length(lanes, map.intersections())
            .unwrap_or(0.0);
        let dist_to_next = startl - mydist;

        for (id, acc, travers_length, _) in std::iter::once((travers.kind, -mydist, startl, startl))
            .chain(traverse_forward(
                map,
                it,
                stop_dist + 15.0,
                dist_to_next,
                -1.0,
            ))
        {
            if let Some(locs) = reservs.localisations.get(&id) {
                for (&train, &otherdist) in locs {
                    if train == me {
                        continue;
                    }
                    if let Some(otherloco) = locoview.get(train) {
                        let dist_to_other = acc + otherdist + travers_length;
                        if dist_to_other > 0.0
                            && dist_to_other < otherloco.length + stop_dist + 10.0
                        {
                            return 0.0;
                        }
                    }
                }
            }
            if let TraverseKind::Turn(id) = id {
                if let Some(inter) = map.intersections().get(id.parent) {
                    if inter.roads.len() > 2 {
                        if let Some(reserved_by) = reservs.reservations.get(&id.parent) {
                            if *reserved_by != me {
                                return 0.0;
                            }
                        } else {
                            return 0.0;
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
