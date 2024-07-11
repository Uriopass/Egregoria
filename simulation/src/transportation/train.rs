use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmapd::HopSlotMap;

use egui_inspect::Inspect;
use geom::{PolyLine3, Polyline3Queue, Transform, Vec3};
use prototypes::{RollingStockID, DELTA};

use crate::map::{IntersectionID, LaneID, Map, TraverseKind};
use crate::map_dynamic::ItineraryFollower;
use crate::transportation::Speed;
use crate::utils::resources::Resources;
use crate::world::{TrainEnt, TrainID, WagonEnt};
use crate::{Itinerary, ItineraryLeader, Simulation, World};

#[derive(Default, Serialize, Deserialize)]
pub struct TrainReservations {
    pub reservations: BTreeMap<IntersectionID, TrainID>,
    pub localisations: BTreeMap<TraverseKind, BTreeMap<TrainID, f32>>,
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Inspect)]
pub enum RailWagonKind {
    Locomotive,
    Passenger,
    Freight,
}

#[derive(Inspect, Serialize, Deserialize)]
pub struct RailWagon {
    pub kind: RailWagonKind,
    pub rolling_stock: RollingStockID,
}

pub fn calculate_locomotive(wagons: &[RollingStockID]) -> Locomotive {
    let info = wagons.iter().fold(
        (720.0, 0.0, 0.0, 0.0, 0),
        |(speed, acc, dec, length, mass): (f32, f32, f32, f32, u32), &id| {
            let rs = RollingStockID::prototype(id);
            (
                speed.min(rs.max_speed),
                acc + rs.acc_force,
                dec + rs.dec_force,
                length + rs.length,
                mass + rs.mass,
            )
        },
    );
    Locomotive {
        max_speed: info.0,
        acc_force: info.1 / info.4 as f32,
        dec_force: info.2 / info.4 as f32,
        length: info.3 + 10.0,
    }
}

pub fn wagons_loco_dists_lengths(
    wagons: &[RollingStockID],
) -> impl DoubleEndedIterator<Item = (f32, f32)> + '_ {
    let mut loco_dist = 0.0;
    wagons.iter().map(move |&id| {
        let length = RollingStockID::prototype(id).length;
        loco_dist += length;
        (loco_dist - length, length)
    })
}

pub fn wagons_positions_for_render<'a>(
    wagons: &'a [RollingStockID],
    points: &'a PolyLine3,
    dist: f32,
) -> impl Iterator<Item = (Vec3, Vec3, f32)> + 'a {
    wagons_loco_dists_lengths(wagons)
        .map(|(wagon_dist, length)| (wagon_dist + length * 0.5, length))
        .rev()
        .filter_map(move |(wagin_dist, length)| {
            let pos = dist - wagin_dist;
            if pos >= 0.0 {
                Some((pos, length))
            } else {
                None
            }
        })
        .map(move |(d, length)| {
            let (pos, dir) = points.point_dir_along(d);
            (pos, dir, length)
        })
}

pub fn train_length(wagons: &[RollingStockID]) -> f32 {
    wagons
        .iter()
        .map(|id| RollingStockID::prototype(*id).length)
        .sum::<f32>()
}

pub fn spawn_train(
    sim: &mut Simulation,
    wagons: &[RollingStockID],
    kind: RailWagonKind,
    lane: LaneID,
    dist: f32,
) -> Option<TrainID> {
    let (world, res) = sim.world_res();

    let map = res.read::<Map>();
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

    let locomotive = calculate_locomotive(wagons);
    let train_length = locomotive.length;

    let loco = world.insert(TrainEnt {
        trans: Transform::new_dir(locopos, locodir),
        speed: Default::default(),
        it: Itinerary::NONE,
        locomotive,
        res: LocomotiveReservation {
            cur_travers_dist: dist,
            waited_for: 0.0,
            past_travers: BTreeMap::from([(
                TraverseKind::Lane(lane.id),
                dist - lane.points.length(),
            )]),
            upcoming_inters: Default::default(),
        },
        leader: ItineraryLeader {
            past: Polyline3Queue::new(points.into_iter(), locopos, train_length + 20.0),
        },
    });

    let leader = &world.trains.get(loco).unwrap().leader;

    let mut followers: Vec<_> = leader
        .past
        .mk_followers(
            wagons_loco_dists_lengths(wagons)
                .flat_map(|(dist, length)| [dist + length * 0.1, dist + length * 0.9]),
        )
        .collect();
    for (i, follower) in followers.chunks_exact_mut(2).enumerate() {
        let (pos, dir) = follower[0].update(&leader.past);
        let (pos2, dir2) = follower[1].update(&leader.past);
        world.wagons.insert(WagonEnt {
            trans: Transform::new_dir(pos * 0.5 + pos2 * 0.5, (0.5 * (dir + dir2)).normalize()),
            speed: Speed::default(),
            wagon: RailWagon {
                rolling_stock: wagons[i],
                kind: if i == 0 {
                    RailWagonKind::Locomotive
                } else {
                    kind
                },
            },
            itfollower: ItineraryFollower {
                leader: loco,
                head: follower[0],
                tail: follower[1],
            },
        });
    }

    log::info!("Spawned Train with {} wagons", wagons.len());
    Some(loco)
}

pub fn traverse_forward<'a>(
    map: &'a Map,
    itin: &'a Itinerary,
    dist: f32,
    mut acc: f32,
    until_length: f32,
) -> impl Iterator<Item = (TraverseKind, f32, f32, f32)> + 'a {
    let route = itin.get_route();
    let lanes = map.lanes();
    let inters = map.intersections();
    let mut acc_inter = 0.0;
    route
        .into_iter()
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

pub fn train_reservations_update(world: &mut World, resources: &mut Resources) {
    profiling::scope!("transportation::train_reservations_update");
    let map = &*resources.read::<Map>();
    let reservations = &mut *resources.write::<TrainReservations>();
    let lanes = map.lanes();
    let inters = map.intersections();
    world.trains.iter_mut().for_each(move |(me, train)| {
        // Remember when we've been
        if let Some(travers) = train.it.get_travers() {
            match train.res.past_travers.entry(travers.kind) {
                Entry::Vacant(v) => {
                    v.insert(10.0 - travers.kind.length(lanes, inters).unwrap_or(0.0));
                    train.res.cur_travers_dist = 0.0;
                }
                Entry::Occupied(_) => {}
            };

            // Handle upcoming intersections
            // Start by cleaning them then re-reserve them (so that in event of weirdness, they stay correct)
            for v in train.res.upcoming_inters.drain(..) {
                reservations.reservations.remove(&v);
            }

            let dist_to_next =
                travers.kind.length(lanes, inters).unwrap_or(0.0) - train.res.cur_travers_dist;

            let mut want_to_reserve = vec![];
            let mut all_ok = true;
            // Then look ahead stop_dist to reserve all intersections
            let stop_dist = train.speed.0 * train.speed.0 / (2.0 * train.locomotive.dec_force);

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
                    &train.it,
                    stop_dist + 5.0,
                    dist_to_next,
                    train.locomotive.length + 25.0,
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
                        train.res.upcoming_inters.push(id);
                    }
                }
            }
        }

        // Clean past_things and unreserve them
        let length = train.locomotive.length;
        train.res.past_travers.retain(|&id, dist| {
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

pub fn locomotive_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("transportation::locomotive_system");
    let map: &Map = &resources.read();
    let reservs: &TrainReservations = &resources.read();

    // asume iter order stays the same
    let mut desired_speeds = Vec::with_capacity(world.trains.len());

    for (ent, train) in world.trains.iter() {
        desired_speeds.push(locomotive_desired_speed(
            ent,
            map,
            reservs,
            &world.trains,
            train,
        ));
    }

    for (t, desired_speed) in world.trains.values_mut().zip(desired_speeds) {
        let desired_dir =
            t.it.get_point()
                .and_then(|x| {
                    let d = x - t.trans.pos;
                    if d.mag2() < 0.5 {
                        return None;
                    }
                    d.try_normalize()
                })
                .unwrap_or(t.trans.dir);
        t.trans.dir = desired_dir;

        t.speed.0 += (desired_speed - t.speed.0).clamp(
            -DELTA * t.locomotive.dec_force,
            DELTA * t.locomotive.acc_force,
        );
        if t.speed.0 <= 0.001 {
            t.res.waited_for += DELTA;
        } else {
            t.res.waited_for = 0.0;
        }
        for v in t.res.past_travers.values_mut() {
            *v += t.speed.0 * DELTA;
            if t.res.waited_for > 60.0 {
                *v += 0.1 * DELTA;
            }
        }
        t.res.cur_travers_dist += t.speed.0 * DELTA;
    }
}

pub fn locomotive_desired_speed(
    me: TrainID,
    map: &Map,
    reservs: &TrainReservations,
    locos: &HopSlotMap<TrainID, TrainEnt>,
    t: &TrainEnt,
) -> f32 {
    if t.it.is_none_or_wait() {
        return 0.0;
    }

    let stop_dist = t.speed.0 * t.speed.0 / (2.0 * t.locomotive.dec_force);

    let mut lastid = None;
    let mydist = t.res.cur_travers_dist;
    if let Some(travers) = t.it.get_travers() {
        let lanes = map.lanes();

        let startl = travers
            .kind
            .length(lanes, map.intersections())
            .unwrap_or(0.0);
        let dist_to_next = startl - mydist;

        for (id, acc, travers_length, _) in std::iter::once((travers.kind, -mydist, startl, startl))
            .chain(traverse_forward(
                map,
                &t.it,
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
                    if let Some(otherloco) = locos.get(train) {
                        let dist_to_other = acc + otherdist + travers_length;
                        if dist_to_other > 0.0
                            && dist_to_other < otherloco.locomotive.length + stop_dist + 10.0
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
    if let Some(r) = t.it.get_route() {
        if r.reversed_route.is_empty()
            || (lastid.is_some() && lastid == r.reversed_route.first().map(|x| x.kind))
        {
            on_last_lane = true;
        }
    }

    if t.it.is_simple() {
        on_last_lane = true
    }

    if on_last_lane {
        if let Some(howfar) = t.it.end_pos().map(|term| term.distance(t.trans.pos)) {
            if howfar + 0.1 <= stop_dist {
                return 0.0;
            }
        }
    }

    t.locomotive.max_speed
}
