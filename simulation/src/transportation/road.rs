use crate::map::{Map, TrafficBehavior, Traversable, TraverseKind};
use crate::map_dynamic::{Itinerary, OBJECTIVE_OK_DIST};
use crate::transportation::{
    Speed, TransportGrid, TransportState, TransportationGroup, Transporter,
};
use crate::transportation::{Vehicle, VehicleState, TIME_TO_PARK};
use crate::utils::resources::Resources;
use crate::world::{VehicleEnt, VehicleID};
use crate::ParCommandBuffer;
use crate::World;
use geom::{angle_lerpxy, Ray, Transform, Vec2, Vec3};
use prototypes::{GameTime, DELTA};
use slotmapd::Key;

pub fn vehicle_decision_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("transportation::vehicle_decision_system");
    let ra = &*resources.read();
    let rb = &*resources.read();
    let rc = &*resources.read();

    world.vehicles.iter_mut().for_each(|(ent, v)| {
        let Some(ref coll) = v.collider else {
            return;
        };

        vehicle_decision(
            ra,
            rb,
            rc,
            ent,
            &mut v.it,
            &mut v.trans,
            &mut v.speed,
            &mut v.vehicle,
            coll,
        );
    });
}

pub fn vehicle_decision(
    map: &Map,
    time: &GameTime,
    cow: &TransportGrid,
    me: VehicleID,
    it: &mut Itinerary,
    trans: &mut Transform,
    kin: &mut Speed,
    vehicle: &mut Vehicle,
    collider: &Transporter,
) {
    let (_, self_obj) = cow.get(collider.0).expect("Handle not in transport grid");

    let mut desired_speed = 0.0;
    let mut desired_dir = Vec3::ZERO;
    if matches!(
        vehicle.state,
        VehicleState::Driving | VehicleState::Panicking(_)
    ) {
        let danger_length =
            (self_obj.speed.powi(2) / (2.0 * vehicle.kind.deceleration())).min(100.0);
        let neighbors = cow.query_around(trans.pos.xy(), 12.0 + danger_length);
        let objs =
            neighbors.map(|(id, pos)| (pos, cow.get(id).expect("Handle not in transport grid").1));

        let (s, d) = calc_decision(me, vehicle, map, time, trans, self_obj, it, objs);
        desired_speed = s;
        desired_dir = d;
    }

    physics(
        trans,
        kin,
        vehicle,
        self_obj,
        map,
        desired_speed,
        desired_dir,
    );
}

pub fn vehicle_state_update_system(world: &mut World, resources: &mut Resources) {
    profiling::scope!("transportation::vehicle_state_update_system");
    let ra = &*resources.read();
    let rb = &*resources.read();

    world.vehicles.iter_mut().for_each(|(ent, v)| {
        vehicle_state_update(
            ra,
            rb,
            ent,
            &mut v.vehicle,
            &mut v.trans,
            &mut v.speed,
            &mut v.collider,
        );
    });
}

/// Decides whether a vehicle should change states, from parked to unparking to driving etc
pub fn vehicle_state_update(
    buf: &ParCommandBuffer<VehicleEnt>,
    map: &Map,
    ent: VehicleID,
    vehicle: &mut Vehicle,
    trans: &mut Transform,
    kin: &mut Speed,
    coll: &mut Option<Transporter>,
) {
    match vehicle.state {
        VehicleState::RoadToPark(_, ref mut t, _) => {
            // Vehicle is on rails when parking.
            *t += DELTA / TIME_TO_PARK;

            if *t >= 1.0 {
                let v = coll.take();
                if let Some(x) = v {
                    buf.exec_ent(ent, x.destroy());
                }
                kin.0 = 0.0;
                let spot = match std::mem::replace(&mut vehicle.state, VehicleState::Driving) {
                    VehicleState::RoadToPark(_, _, spot) => spot,
                    _ => unreachable!(),
                };
                vehicle.state = VehicleState::Parked(spot);
            }
        }
        VehicleState::Parked(ref spot) => {
            if let Some(p) = spot.get(&map.parking) {
                if p.trans != *trans {
                    *trans = p.trans;
                }
            } else {
                buf.kill(ent);
            }
        }
        _ => {}
    }
}

/// Handles actually moving the vehicles around, including acceleration and other physics stuff.
fn physics(
    trans: &mut Transform,
    kin: &mut Speed,
    vehicle: &mut Vehicle,
    obj: &TransportState,
    map: &Map,
    desired_speed: f32,
    desired_dir: Vec3,
) {
    match vehicle.state {
        VehicleState::Parked(ref id) => {
            let spot = unwrap_ret!(id.get(&map.parking));
            *trans = spot.trans;
            return;
        }
        VehicleState::RoadToPark(spline, t, _) => {
            trans.pos = spline.get(t);
            trans.dir = spline.derivative(t).normalize();
            return;
        }
        _ => {}
    }

    let speed = obj.speed;
    let kind = vehicle.kind;

    let speed = speed
        + (desired_speed - speed).clamp(-DELTA * kind.deceleration(), DELTA * kind.acceleration());

    let max_ang_vel = (speed.abs() / kind.min_turning_radius()).clamp(0.0, 3.0);

    let approx_angle = trans.dir.distance(desired_dir);

    vehicle.ang_velocity += DELTA * kind.ang_acc();
    vehicle.ang_velocity = vehicle
        .ang_velocity
        .min(4.0 * approx_angle)
        .min(max_ang_vel);

    trans.dir = angle_lerpxy(trans.dir, desired_dir, vehicle.ang_velocity * DELTA);

    kin.0 = speed;
}

/// Decide the appropriate velocity and direction to aim for.
pub fn calc_decision<'a>(
    me: VehicleID,
    vehicle: &mut Vehicle,
    map: &Map,
    time: &GameTime,
    trans: &Transform,
    self_obj: &TransportState,
    it: &Itinerary,
    neighs: impl Iterator<Item = (Vec2, &'a TransportState)>,
) -> (f32, Vec3) {
    let default_return = (0.0, trans.dir);
    if vehicle.wait_time > 0.0 {
        vehicle.wait_time -= DELTA;
        return default_return;
    }
    let objective: Vec3 = unwrap_or!(it.get_point(), return default_return);

    let speed = self_obj.speed;
    let time_to_stop = speed / vehicle.kind.deceleration();
    let stop_dist = time_to_stop * speed * 0.5;

    let cutoff = (0.8 + stop_dist).min(1.5);

    let (front_dist, flag) = calc_front_dist(vehicle, trans, self_obj, it, neighs, cutoff);

    let position = trans.pos;
    let dir_to_pos = unwrap_or!(
        (objective - position).try_normalize(),
        return default_return
    );

    if let VehicleState::Panicking(since) = vehicle.state {
        if since.elapsed(time).seconds() > 200.0 {
            vehicle.state = VehicleState::Driving;
        }
    } else if speed.abs() < 0.2 && front_dist < 1.5 {
        let me_u64: u64 = me.data().as_ffi();
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

    if let Some(term_pos) = it.get_terminal() {
        if term_pos.is_close(position, stop_dist) {
            return (0.0, dir_to_pos);
        }
    }

    let mut speed = 9.0;

    if let Some(Traversable {
        kind: TraverseKind::Lane(l_id),
        ..
    }) = it.get_travers()
    {
        if let Some(l) = map.lanes().get(*l_id) {
            speed = l.speed_limit;

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
                    if light.is_close_signed(position, stop_dist) && !light.is_close(position, 1.0)
                    {
                        return (0.0, dir_to_pos);
                    }
                }
                TrafficBehavior::GREEN => {
                    if light.is_close(position, stop_dist * 0.4) {
                        return (0.0, dir_to_pos);
                    }
                }
            }
        }
    }

    // Not facing the objective
    if dir_to_pos.dot(trans.dir) < 0.8 {
        return (6.0, dir_to_pos);
    }

    (
        vehicle.kind.speed_factor() * vehicle.max_speed_multiplier * speed,
        dir_to_pos,
    )
}

/// Calculates the distance to the closest problematic object in front of the car.
/// It can be another car or a pedestrian, or it can be a potential collision point from a
/// car coming perpendicularly.
fn calc_front_dist<'a>(
    vehicle: &mut Vehicle,
    trans: &Transform,
    self_obj: &TransportState,
    it: &Itinerary,
    neighs: impl Iterator<Item = (Vec2, &'a TransportState)>,
    cutoff: f32,
) -> (f32, u64) {
    let position = trans.pos;
    let direction = trans.dir;
    let pos2 = position.xy();
    let dir2 = trans.dir.xy();

    let mut min_front_dist: f32 = 50.0;

    let my_ray = Ray {
        from: position.xy() - direction.xy() * vehicle.kind.width() * 0.5,
        dir: direction.xy(),
    };

    let my_radius = self_obj.radius;
    let speed = self_obj.speed;

    let on_lane = it.get_travers().map_or(false, |t| t.kind.is_lane());
    let mut flag = 0;
    // Collision avoidance
    for (his_pos, nei_physics_obj) in neighs {
        if (nei_physics_obj.height - position.z).abs() > 5.0 {
            continue;
        }
        let towards_vec: Vec2 = his_pos - pos2;
        // Ignore myself and very close cars
        if towards_vec.is_close(Vec2::ZERO, 1.0) && towards_vec.x > 0.0 {
            continue;
        }

        let (towards_dir, dist) = unwrap_or!(towards_vec.dir_dist(), continue);

        // cos of angle from self to obj
        let cos_angle = towards_dir.dot(dir2);

        // Ignore things behind
        if cos_angle < 0.0 {
            continue;
        }

        let dist_to_side = towards_vec.perp_dot(dir2).abs();

        let is_vehicle = matches!(nei_physics_obj.group, TransportationGroup::Vehicles);

        let cos_direction_angle = nei_physics_obj.dir.dot(dir2);

        // front cone
        if cos_angle > 0.85 - 0.015 * speed.min(10.0)
            && (!is_vehicle || cos_direction_angle > 0.0)
            && (!on_lane || dist_to_side < 3.0 + speed * 0.3)
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

        let (my_dist, his_dist) = unwrap_or!(my_ray.both_dist_to_inter(&his_ray), continue);

        if my_dist.max(his_dist) > 1000.0 {
            continue;
        }

        if nei_physics_obj.speed <= 0.01 {
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
