use crate::map_dynamic::{Itinerary, ParkingManagement, OBJECTIVE_OK_DIST};
use crate::physics::Kinematics;
use crate::physics::{Collider, CollisionWorld, PhysicsGroup, PhysicsObject};
use crate::utils::Restrict;
use crate::vehicles::{Vehicle, VehicleState, TIME_TO_PARK};
use crate::{Deleted, ParCommandBuffer};
use common::GameTime;
use geom::{angle_lerp, Transform, Vec2};
use geom::{both_dist_to_inter, Ray};
use legion::system;
use legion::Entity;
use map_model::{Map, TrafficBehavior, Traversable, TraverseKind};

#[system]
pub fn vehicle_cleanup(
    #[resource] evts: &mut Deleted<Vehicle>,
    #[resource] pm: &mut ParkingManagement,
) {
    for comp in evts.drain() {
        if let VehicleState::Parked(id) | VehicleState::RoadToPark(_, _, id) = comp.state {
            pm.free(id)
        }
    }
}

#[system(par_for_each)]
pub fn vehicle_decision(
    #[resource] map: &Map,
    #[resource] time: &GameTime,
    #[resource] cow: &CollisionWorld,
    me: &Entity,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Kinematics,
    vehicle: &mut Vehicle,
    collider: &Collider,
) {
    let (_, self_obj) = cow.get(collider.0).expect("Handle not in collision world");

    let mut desired_speed = 0.0;
    let mut desired_dir = Vec2::ZERO;
    if matches!(vehicle.state, VehicleState::Driving | VehicleState::Panicking(_)) {
        let danger_length =
            (self_obj.speed.powi(2) / (2.0 * vehicle.kind.deceleration())).min(40.0);
        let neighbors = cow.query_around(trans.position(), 12.0 + danger_length);
        let objs =
            neighbors.map(|(id, pos)| (pos, cow.get(id).expect("Handle not in collision world").1));

        let (s, d) = calc_decision(*me, vehicle, &map, &time, trans, self_obj, it, objs);
        desired_speed = s;
        desired_dir = d;
    }

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
}

/// Decides whether a vehicle should change states, from parked to unparking to driving etc
#[system(for_each)]
pub fn vehicle_state_update(
    #[resource] buf: &ParCommandBuffer,
    #[resource] time: &GameTime,
    vehicle: &mut Vehicle,
    kin: &mut Kinematics,
    ent: &Entity,
) {
    if let VehicleState::RoadToPark(_, ref mut t, spot) = vehicle.state {
        // Vehicle is on rails when parking.
        *t += time.delta / TIME_TO_PARK;

        if *t >= 1.0 {
            buf.remove_component::<Collider>(*ent);
            kin.velocity = Vec2::ZERO;
            vehicle.state = VehicleState::Parked(spot);
        }
    }
}

/// Handles actually moving the vehicles around, including acceleration and other physics stuff.
fn physics(
    trans: &mut Transform,
    kin: &mut Kinematics,
    vehicle: &mut Vehicle,
    time: &GameTime,
    obj: &PhysicsObject,
    map: &Map,
    desired_speed: f32,
    desired_dir: Vec2,
) {
    match vehicle.state {
        VehicleState::Parked(id) => {
            let spot = unwrap_or!(map.parking.get(id), return);
            *trans = spot.trans;
            return;
        }
        VehicleState::RoadToPark(spline, t, _) => {
            trans.set_position(spline.get(t));
            trans.set_direction(spline.derivative(t).normalize());
            return;
        }
        _ => {}
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

/// Decide the appropriate velocity and direction to aim for.
pub fn calc_decision<'a>(
    me: Entity,
    vehicle: &mut Vehicle,
    map: &Map,
    time: &GameTime,
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

    let speed = self_obj.speed;
    let time_to_stop = speed / vehicle.kind.deceleration();
    let stop_dist = time_to_stop * speed * 0.5;

    let cutoff = (0.8 + stop_dist).min(1.5);

    let (front_dist, flag) = calc_front_dist(vehicle, trans, self_obj, it, neighs, cutoff);

    let position = trans.position();
    let dir_to_pos = unwrap_or!(
        (objective - position).try_normalize(),
        return default_return
    );

    if let VehicleState::Panicking(since) = vehicle.state {
        if since.elapsed(time) > 5.0 {
            vehicle.state = VehicleState::Driving;
        }
    } else if speed.abs() < 0.2 && front_dist < 1.5 {
        let me_u64: u64 = unsafe { std::mem::transmute(me) };
        if me_u64 == flag {
            vehicle.state = VehicleState::Panicking(time.instant());
            log::info!("gridlock!")
        }
        vehicle.flag = if vehicle.flag | flag == 0 {
            me_u64
        } else {
            flag
        };
        vehicle.wait_time = (position.x * 1000.0).fract().abs() * 0.5;
        return default_return;
    } else {
        // Stop at 80 cm of object in front
        if front_dist < 0.8 + stop_dist {
            return (0.0, dir_to_pos);
        }
    }

    vehicle.flag = 0;

    if let Some(pos) = terminal_pos {
        if pos.is_close(trans.position(), 1.0 + stop_dist) {
            return (0.0, dir_to_pos);
        }
    }

    if let Some(Traversable {
        kind: TraverseKind::Lane(l_id),
        ..
    }) = it.get_travers()
    {
        if let Some(l) = map.lanes().get(*l_id) {
            let light = l.control_point();
            match l.control.get_behavior(time.seconds) {
                TrafficBehavior::RED | TrafficBehavior::ORANGE => {
                    if light.is_close(
                        position,
                        OBJECTIVE_OK_DIST * 1.05
                            + 2.0
                            + stop_dist
                            + (vehicle.kind.width() * 0.5 - OBJECTIVE_OK_DIST).max(0.0),
                    ) {
                        return (0.0, dir_to_pos);
                    }
                }
                TrafficBehavior::STOP => {
                    if light.is_close(position, OBJECTIVE_OK_DIST * 0.95 + stop_dist) {
                        return (0.0, dir_to_pos);
                    }
                }
                _ => {}
            }
        }
    }

    // Not facing the objective
    if dir_to_pos.dot(trans.direction()) < 0.8 {
        return (6.0, dir_to_pos);
    }

    (vehicle.kind.cruising_speed(), dir_to_pos)
}

/// Calculates the distance to the closest problematic object in front of the car.
/// It can be another car or a pedestrian, or it can be a potential collision point from a
/// car coming perpendicularly.
fn calc_front_dist<'a>(
    vehicle: &mut Vehicle,
    trans: &Transform,
    self_obj: &PhysicsObject,
    it: &Itinerary,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
    cutoff: f32,
) -> (f32, u64) {
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
    let mut flag = 0;
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
            if dist_to_obj < min_front_dist {
                min_front_dist = dist_to_obj;
                flag = nei_physics_obj.flag;
            }
            if min_front_dist < cutoff {
                return (min_front_dist, flag);
            }
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

        if my_dist.max(his_dist) > 1000.0 {
            continue;
        }

        if my_dist - speed.min(2.5) - my_radius
            < his_dist - nei_physics_obj.speed.min(2.5) - nei_physics_obj.radius
        {
            continue;
        }

        let final_dist = dist - my_radius - nei_physics_obj.radius - 5.0;
        if final_dist < min_front_dist {
            min_front_dist = final_dist;
            flag = nei_physics_obj.flag;
        }
    }
    (min_front_dist, flag)
}
