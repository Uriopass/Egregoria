use crate::engine_interaction::TimeInfo;
use crate::geometry::intersections::{both_dist_to_inter, Ray};
use crate::geometry::{Vec2, Vec2Impl};
use crate::map_model::{
    DirectionalPath, Itinerary, LaneKind, Map, TrafficBehavior, Traversable, TraverseDirection,
    TraverseKind,
};
use crate::physics::{Collider, CollisionWorld, PhysicsGroup, PhysicsObject};
use crate::physics::{Kinematics, Transform};
use crate::utils::{rand_det, Restrict};
use crate::vehicles::VehicleComponent;
use cgmath::{Angle, InnerSpace, MetricSpace, Vector2};
use specs::prelude::*;
use specs::shred::PanicHandler;

#[derive(Default)]
pub struct VehicleDecision;

pub const OBJECTIVE_OK_DIST: f32 = 4.0;

#[derive(SystemData)]
pub struct VehicleDecisionSystemData<'a> {
    map: Read<'a, Map>,
    time: Read<'a, TimeInfo>,
    coworld: Read<'a, CollisionWorld, PanicHandler>,
    colliders: ReadStorage<'a, Collider>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    vehicles: WriteStorage<'a, VehicleComponent>,
}

impl<'a> System<'a> for VehicleDecision {
    type SystemData = VehicleDecisionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let cow = data.coworld;
        let map = &*data.map;
        let time = data.time;

        (
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.vehicles,
            &data.colliders,
        )
            .join()
            .for_each(|(trans, kin, vehicle, collider)| {
                objective_update(vehicle, &time, trans, &map);

                let self_obj = cow.get_obj(collider.0);
                let speed: f32 = self_obj.speed;
                let danger_length = (speed * speed / (2.0 * vehicle.kind.deceleration())).min(40.0);
                let neighbors = cow.query_around(trans.position(), 12.0 + danger_length);
                let objs = neighbors.map(|obj| (obj.pos, cow.get_obj(obj.id)));

                calc_decision(vehicle, map, speed, &time, trans, self_obj, objs);

                vehicle_physics(&time, trans, kin, vehicle, speed);
            });
    }
}

fn vehicle_physics(
    time: &TimeInfo,
    trans: &mut Transform,
    kin: &mut Kinematics,
    vehicle: &mut VehicleComponent,
    speed: f32,
) {
    let kind = vehicle.kind;
    let direction = trans.direction();

    let speed = speed
        + (vehicle.desired_speed - speed).restrict(
            -time.delta * kind.deceleration(),
            time.delta * kind.acceleration(),
        );

    let max_ang_vel = (speed.abs() / kind.min_turning_radius()).restrict(0.0, 2.0);

    let delta_ang = direction.angle(vehicle.desired_dir);
    let mut ang = vec2!(1.0, 0.0).angle(direction);

    vehicle.ang_velocity += time.delta * kind.ang_acc();
    vehicle.ang_velocity = vehicle
        .ang_velocity
        .min(3.0 * delta_ang.0.abs())
        .min(max_ang_vel);

    ang.0 += delta_ang.0.restrict(
        -vehicle.ang_velocity * time.delta,
        vehicle.ang_velocity * time.delta,
    );

    let direction = vec2!(ang.cos(), ang.sin());
    trans.set_direction(direction);

    kin.velocity = direction * speed;
}

pub fn objective_update(
    vehicle: &mut VehicleComponent,
    time: &TimeInfo,
    trans: &Transform,
    map: &Map,
) {
    vehicle.itinerary.check_validity(map);

    let mut last_travers = vehicle.itinerary.get_travers().copied();

    if let Some(p) = vehicle.itinerary.get_point() {
        if p.distance2(trans.position()) < OBJECTIVE_OK_DIST * OBJECTIVE_OK_DIST {
            let k = vehicle.itinerary.get_travers().unwrap();
            if vehicle.itinerary.remaining_points() > 1
                || k.can_pass(time.time_seconds, map.lanes())
            {
                vehicle.itinerary.advance(map);
            }
        }
    }

    if vehicle.itinerary.has_ended(time.time) {
        if last_travers.is_none() {
            last_travers = map
                .closest_lane(trans.position(), LaneKind::Driving)
                .map(|x| Traversable::new(TraverseKind::Lane(x), TraverseDirection::Forward));
        }

        let l = unwrap_or!(map.get_random_lane(LaneKind::Driving), return);

        vehicle.itinerary = Itinerary::route(
            unwrap_or!(last_travers, return),
            (l.id, l.points.random_along().unwrap()),
            map,
            &DirectionalPath,
        );

        if vehicle.itinerary.is_none() {
            println!("No path from {:?} to {:?}", last_travers, l.id);
            vehicle.itinerary = Itinerary::wait_until(time.time + 10.0);
        }
    }
}

pub fn calc_decision<'a>(
    vehicle: &mut VehicleComponent,
    map: &Map,
    speed: f32,
    time: &TimeInfo,
    trans: &Transform,
    self_obj: &PhysicsObject,
    neighs: impl Iterator<Item = (Vec2, &'a PhysicsObject)>,
) {
    vehicle.desired_speed = 0.0;

    if vehicle.wait_time > 0.0 {
        vehicle.wait_time -= time.delta;
        return;
    }
    let objective: Vec2 = unwrap_or!(vehicle.itinerary.get_point(), return);

    let is_terminal = false; // TODO: change depending on route

    let front_dist = calc_front_dist(vehicle, speed, trans, self_obj, neighs);

    if speed.abs() < 0.2 && front_dist < 1.5 {
        vehicle.wait_time = rand_det::<f32>() * 0.5;
        return;
    }

    let delta_pos: Vec2 = objective - trans.position();
    let (dir_to_pos, dist_to_pos) = unwrap_or!(delta_pos.dir_dist(), return);
    let time_to_stop = speed / vehicle.kind.deceleration();
    let stop_dist = time_to_stop * speed / 2.0;

    vehicle.desired_dir = dir_to_pos;
    vehicle.desired_speed = vehicle.kind.cruising_speed();

    if vehicle.itinerary.remaining_points() == 1 {
        if let Some(Traversable {
            kind: TraverseKind::Lane(l_id),
            ..
        }) = vehicle.itinerary.get_travers()
        {
            match map.lanes()[*l_id].control.get_behavior(time.time_seconds) {
                TrafficBehavior::RED | TrafficBehavior::ORANGE => {
                    if dist_to_pos
                        < OBJECTIVE_OK_DIST * 1.05
                            + stop_dist
                            + (vehicle.kind.width() / 2.0 - OBJECTIVE_OK_DIST).max(0.0)
                    {
                        vehicle.desired_speed = 0.0;
                    }
                }
                TrafficBehavior::STOP => {
                    if dist_to_pos < OBJECTIVE_OK_DIST * 0.95 + stop_dist {
                        vehicle.desired_speed = 0.0;
                    }
                }
                _ => {}
            }
        }
    }

    // Not facing the objective
    if dir_to_pos.dot(trans.direction()) < 0.8 {
        vehicle.desired_speed = vehicle.desired_speed.min(6.0);
    }

    // Close to terminal objective
    if is_terminal && dist_to_pos < 1.0 + stop_dist {
        vehicle.desired_speed = 0.0;
    }

    // Stop at 80 cm of object in front
    if front_dist < 0.8 + stop_dist {
        vehicle.desired_speed = 0.0;
    }
}

fn calc_front_dist<'a>(
    vehicle: &VehicleComponent,
    speed: f32,
    trans: &Transform,
    self_obj: &PhysicsObject,
    neighs: impl Iterator<Item = (Vector2<f32>, &'a PhysicsObject)>,
) -> f32 {
    let position = trans.position();
    let direction = trans.direction();
    let direction_normal = trans.normal();

    let mut min_front_dist: f32 = 50.0;

    let my_ray = Ray {
        from: position - direction * vehicle.kind.width() / 2.0,
        dir: direction,
    };

    let my_radius = self_obj.radius;
    let on_lane = vehicle.itinerary.get_travers().unwrap().kind.is_lane();

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

        let dist_to_side = towards_vec.dot(direction_normal).abs();

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
