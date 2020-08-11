use crate::engine_interaction::TimeInfo;
use crate::frame_log::FrameLog;
use crate::map_interaction::{Itinerary, ParkingManagement, OBJECTIVE_OK_DIST};
use crate::physics::{Collider, CollisionWorld, PhysicsGroup, PhysicsObject};
use crate::physics::{Kinematics, Transform};
use crate::utils::Restrict;
use crate::vehicles::{VehicleComponent, VehicleState, TIME_TO_PARK};
use geom::intersections::{both_dist_to_inter, Ray};
use geom::splines::Spline;
use geom::{angle_lerp, Vec2};
use map_model::{
    DirectionalPath, LaneKind, Map, ParkingSpotID, TrafficBehavior, Traversable, TraverseDirection,
    TraverseKind,
};
use rand::thread_rng;
use specs::prelude::*;
use specs::shred::PanicHandler;
use std::sync::Mutex;

#[derive(Default)]
pub struct VehicleDecision;

#[derive(SystemData)]
pub struct VehicleDecisionSystemData<'a> {
    entities: Entities<'a>,
    map: Read<'a, Map>,
    time: Read<'a, TimeInfo>,
    parking: Read<'a, ParkingManagement>,
    flog: Read<'a, FrameLog>,
    coworld: Write<'a, CollisionWorld, PanicHandler>,
    colliders: WriteStorage<'a, Collider>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    vehicles: WriteStorage<'a, VehicleComponent>,
    itinerarys: WriteStorage<'a, Itinerary>,
}

impl<'a> System<'a> for VehicleDecision {
    type SystemData = VehicleDecisionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        time_it!(data.flog, "Vehicle update");

        let mut cow = data.coworld;
        let map = data.map;
        let time = data.time;
        let parking = data.parking;

        {
            let colliders = Mutex::new(&mut data.colliders);
            let cowtex = Mutex::new(&mut *cow);

            (
                &data.transforms,
                &mut data.kinematics,
                &mut data.vehicles,
                &mut data.itinerarys,
                &data.entities,
            )
                .par_join()
                .for_each(|(trans, kin, vehicle, it, ent)| {
                    state_update(
                        vehicle, kin, it, &cowtex, &colliders, ent, &parking, trans, &map, &time,
                    );
                });
        }

        (
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.vehicles,
            &data.itinerarys,
            &data.colliders,
        )
            .par_join()
            .for_each(|(trans, kin, vehicle, it, collider)| {
                let (_, self_obj) = cow.get(collider.0).expect("Handle not in collision world");
                let danger_length =
                    (self_obj.speed.powi(2) / (2.0 * vehicle.kind.deceleration())).min(40.0);
                let neighbors = cow.query_around(trans.position(), 12.0 + danger_length);
                let objs = neighbors.map(|(id, pos)| {
                    (
                        Vec2::from(pos),
                        cow.get(id).expect("Handle not in collision world").1,
                    )
                });

                let (desired_speed, desired_dir) =
                    calc_decision(vehicle, &map, &time, trans, self_obj, it, objs);

                physics(
                    trans,
                    kin,
                    vehicle,
                    &time,
                    self_obj,
                    &map,
                    desired_speed,
                    desired_dir,
                );
            });
    }
}

fn state_update(
    vehicle: &mut VehicleComponent,
    kin: &mut Kinematics,
    it: &mut Itinerary,
    cow: &Mutex<&mut CollisionWorld>,
    colliders: &Mutex<&mut WriteStorage<Collider>>,
    ent: Entity,
    parking: &ParkingManagement,
    trans: &Transform,
    map: &Map,
    time: &TimeInfo,
) {
    match vehicle.state {
        VehicleState::ParkedToRoad(_, ref mut t) => {
            *t += time.delta / TIME_TO_PARK;

            if *t >= 1.0 {
                kin.velocity = trans.direction() * 2.0;

                vehicle.state = VehicleState::Driving;
            }
        }
        VehicleState::RoadToPark(_, ref mut t) => {
            *t += time.delta / TIME_TO_PARK;

            if *t >= 1.0 {
                let spot = unwrap_or!(vehicle.park_spot, {
                    vehicle.state = VehicleState::Driving;
                    return;
                });
                {
                    let mut colliders = colliders.lock().unwrap();
                    let h = colliders.get(ent).expect("Driving car has no collider");
                    cow.lock().unwrap().remove(h.0);
                    colliders.remove(ent);
                }
                kin.velocity = Vec2::ZERO;

                vehicle.state = VehicleState::Parked(spot);
            }
        }
        VehicleState::Driving => {
            if it.has_ended(time.time) {
                *it = Itinerary::wait_until(time.time + 20.0);
                let spot = vehicle.park_spot.and_then(|id| map.parking.get(id));

                let spot = unwrap_or!(spot, return);

                let s = Spline {
                    from: trans.position(),
                    to: spot.pos,
                    from_derivative: trans.direction() * 2.0,
                    to_derivative: spot.orientation * 2.0,
                };

                vehicle.state = VehicleState::RoadToPark(s, 0.0);
                kin.velocity = Vec2::ZERO;
            }
        }
        VehicleState::Parked(spot) => {
            if it.has_ended(time.time) {
                let mut lane = map.parking_to_drive(spot);

                if lane.is_none() {
                    lane = map.closest_lane(trans.position(), LaneKind::Driving);
                }

                let travers: Option<Traversable> = lane
                    .map(|x| Traversable::new(TraverseKind::Lane(x), TraverseDirection::Forward));

                if let Some((itin, park)) =
                    next_objective(trans.position(), parking, map, travers.as_ref())
                {
                    parking.free(spot);

                    let points = itin.get_travers().unwrap().points(map); // Unwrap ok: just got itinerary
                    let d = points.distance_along(points.project(trans.position()));

                    let (pos, dir) = points.point_dir_along(d + 5.0);

                    let s = Spline {
                        from: trans.position(),
                        to: pos,
                        from_derivative: trans.direction() * 2.0,
                        to_derivative: dir * 2.0,
                    };

                    let h = Collider(cow.lock().unwrap().insert(
                        trans.position(),
                        PhysicsObject {
                            dir: trans.direction(),
                            group: PhysicsGroup::Vehicles,
                            radius: vehicle.kind.width() * 0.5,
                            speed: 0.0,
                        },
                    ));
                    colliders
                        .lock()
                        .unwrap()
                        .insert(ent, h)
                        .expect("Invalid entity ?");

                    *it = itin;
                    vehicle.park_spot = Some(park);
                    vehicle.state = VehicleState::ParkedToRoad(s, 0.0);
                } else {
                    *it = Itinerary::wait_until(time.time + 10.0);
                }
            }
        }
    }
}

fn physics(
    trans: &mut Transform,
    kin: &mut Kinematics,
    vehicle: &mut VehicleComponent,
    time: &TimeInfo,
    obj: &PhysicsObject,
    map: &Map,
    desired_speed: f32,
    desired_dir: Vec2,
) {
    match vehicle.state {
        VehicleState::Parked(id) => {
            let spot = unwrap_or!(map.parking.get(id), return);
            trans.set_position(spot.pos);
            trans.set_direction(spot.orientation);
            return;
        }
        VehicleState::ParkedToRoad(spline, t) | VehicleState::RoadToPark(spline, t) => {
            trans.set_position(spline.get(t));
            trans.set_direction(spline.derivative(t).normalize());
            return;
        }
        VehicleState::Driving => {}
    }

    let speed = obj.speed;
    let kind = vehicle.kind;
    let direction = trans.direction();

    let speed = speed
        + (desired_speed - speed).restrict(
            -time.delta * kind.deceleration(),
            time.delta * kind.acceleration(),
        );

    let max_ang_vel = (speed.abs() / kind.min_turning_radius()).restrict(0.0, 2.0);

    let approx_angle = direction.distance(desired_dir);

    vehicle.ang_velocity += time.delta * kind.ang_acc();
    vehicle.ang_velocity = vehicle
        .ang_velocity
        .min(3.0 * approx_angle)
        .min(max_ang_vel);

    trans.set_direction(angle_lerp(
        trans.direction(),
        desired_dir,
        vehicle.ang_velocity * time.delta,
    ));

    kin.velocity = trans.direction() * speed;
}

fn next_objective(
    pos: Vec2,
    parking: &ParkingManagement,
    map: &Map,
    last_travers: Option<&Traversable>,
) -> Option<(Itinerary, ParkingSpotID)> {
    let rlane = map.get_random_lane(LaneKind::Driving, &mut thread_rng())?;
    let spot_id = parking.reserve_near(
        rlane.id,
        rlane
            .points
            .point_along(rand::random::<f32>() * rlane.points.length()),
        map,
    )?;

    let l = &map.lanes()[map.parking_to_drive(spot_id)?];

    let spot = map.parking.get(spot_id).unwrap(); // Unwrap ok: gotten using reserve_near

    let p = l.points.project(spot.pos);
    let dist = l.points.distance_along(p);

    Itinerary::route(
        pos,
        *last_travers.filter(|t| t.is_valid(map))?,
        (l.id, l.points.point_along(dist - 5.0)),
        map,
        &DirectionalPath,
    )
    .map(move |it| (it, spot_id))
}

pub fn calc_decision<'a>(
    vehicle: &mut VehicleComponent,
    map: &Map,
    time: &TimeInfo,
    trans: &Transform,
    self_obj: &PhysicsObject,
    it: &Itinerary,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
) -> (f32, Vec2) {
    let default_return = (0.0, self_obj.dir);
    if vehicle.wait_time > 0.0 {
        vehicle.wait_time -= time.delta;
        return default_return;
    }
    let objective: Vec2 = unwrap_or!(it.get_point(), return default_return);

    let terminal_pos = it.get_terminal();

    let front_dist = calc_front_dist(vehicle, trans, self_obj, it, neighs);

    let position = trans.position();
    let speed = self_obj.speed;
    if speed.abs() < 0.2 && front_dist < 1.5 {
        vehicle.wait_time = (position.x * 1000.0).fract().abs() * 0.5;
        return default_return;
    }

    let dir_to_pos = unwrap_or!(
        (objective - position).try_normalize(),
        return default_return
    );

    let time_to_stop = speed / vehicle.kind.deceleration();
    let stop_dist = time_to_stop * speed * 0.5;

    if let Some(pos) = terminal_pos {
        // Close to terminal objective
        if pos.distance(trans.position()) < 1.0 + stop_dist {
            return (0.0, dir_to_pos);
        }
    }

    if let Some(Traversable {
        kind: TraverseKind::Lane(l_id),
        ..
    }) = it.get_travers()
    {
        if let Some(l) = map.lanes().get(*l_id) {
            let dist_to_light = l.control_point().distance(position);
            match l.control.get_behavior(time.time_seconds) {
                TrafficBehavior::RED | TrafficBehavior::ORANGE => {
                    if dist_to_light
                        < OBJECTIVE_OK_DIST * 1.05
                            + 2.0
                            + stop_dist
                            + (vehicle.kind.width() * 0.5 - OBJECTIVE_OK_DIST).max(0.0)
                    {
                        return (0.0, dir_to_pos);
                    }
                }
                TrafficBehavior::STOP => {
                    if dist_to_light < OBJECTIVE_OK_DIST * 0.95 + stop_dist {
                        return (0.0, dir_to_pos);
                    }
                }
                _ => {}
            }
        }
    }

    // Stop at 80 cm of object in front
    if front_dist < 0.8 + stop_dist {
        return (0.0, dir_to_pos);
    }

    // Not facing the objective
    if dir_to_pos.dot(trans.direction()) < 0.8 {
        return (6.0, dir_to_pos);
    }

    (vehicle.kind.cruising_speed(), dir_to_pos)
}

fn calc_front_dist<'a>(
    vehicle: &mut VehicleComponent,
    trans: &Transform,
    self_obj: &PhysicsObject,
    it: &Itinerary,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
) -> f32 {
    let position = trans.position();
    let direction = trans.direction();

    let mut min_front_dist: f32 = 50.0;

    let my_ray = Ray {
        from: position - direction * vehicle.kind.width() * 0.5,
        dir: direction,
    };

    let my_radius = self_obj.radius;
    let speed = self_obj.speed;

    let on_lane = it.get_travers().map_or(false, |t| t.kind.is_lane());

    // Collision avoidance
    for (his_pos, nei_physics_obj) in neighs {
        // Ignore myself
        if std::ptr::eq(nei_physics_obj, self_obj) {
            continue;
        }

        let towards_vec: Vec2 = his_pos - position;
        let (towards_dir, dist) = unwrap_or!(towards_vec.dir_dist(), continue);

        // cos of angle from self to obj
        let cos_angle = towards_dir.dot(direction);

        // Ignore things behind
        if cos_angle < 0.0 {
            continue;
        }

        let dist_to_side = towards_vec.perp_dot(direction).abs();

        let is_vehicle = matches!(nei_physics_obj.group, PhysicsGroup::Vehicles);

        let cos_direction_angle = nei_physics_obj.dir.dot(direction);

        // front cone
        if cos_angle > 0.85 - 0.015 * speed.min(10.0)
            && (!is_vehicle || cos_direction_angle > 0.0)
            && (!on_lane || dist_to_side < 3.0)
        {
            let mut dist_to_obj = dist - my_radius - nei_physics_obj.radius;
            if !is_vehicle {
                dist_to_obj -= 1.0;
            }
            min_front_dist = min_front_dist.min(dist_to_obj);
            continue;
        }

        // don't do ray checks for other things than cars
        if !is_vehicle {
            continue;
        }

        // closest win
        let his_ray = Ray {
            from: his_pos - nei_physics_obj.radius * nei_physics_obj.dir,
            dir: nei_physics_obj.dir,
        };

        let (my_dist, his_dist) = unwrap_or!(both_dist_to_inter(my_ray, his_ray), continue);

        if my_dist - speed.min(2.5) - my_radius
            < his_dist - nei_physics_obj.speed.min(2.5) - nei_physics_obj.radius
        {
            continue;
        }

        min_front_dist = min_front_dist.min(dist - my_radius - nei_physics_obj.radius - 5.0);
    }
    min_front_dist
}
