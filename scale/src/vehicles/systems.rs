use crate::engine_interaction::TimeInfo;
use crate::geometry::intersections::{both_dist_to_inter, Ray};
use crate::map_model::{Map, TrafficBehavior, Traversable, Turn, TurnID};
use crate::physics::{CollisionWorld, PhysicsObject};
use crate::physics::{Kinematics, Transform};
use crate::vehicles::VehicleComponent;
use crate::vehicles::VehicleObjective;
use crate::vehicles::VehicleObjective::Temporary;
use cgmath::{vec2, Angle, InnerSpace, MetricSpace, Vector2};
use specs::prelude::*;
use specs::shred::PanicHandler;

#[derive(Default)]
pub struct VehicleDecision;

pub const OBJECTIVE_OK_DIST: f32 = 4.0;

#[derive(SystemData)]
pub struct VehicleDecisionSystemData<'a> {
    map: Read<'a, Map, PanicHandler>,
    time: Read<'a, TimeInfo>,
    coworld: Read<'a, CollisionWorld, PanicHandler>,
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
        )
            .join()
            .for_each(|(trans, kin, vehicle)| {
                objective_update(vehicle, &time, trans, &map);
                vehicle_physics(&cow, &map, &time, trans, kin, vehicle);
            });
    }
}

fn vehicle_physics(
    coworld: &CollisionWorld,
    map: &Map,
    time: &TimeInfo,
    trans: &mut Transform,
    kin: &mut Kinematics,
    vehicle: &mut VehicleComponent,
) {
    let direction = trans.direction();
    let speed: f32 = kin.velocity.magnitude() * kin.velocity.dot(direction).signum();
    let dot = (kin.velocity / speed).dot(direction);
    let kind = vehicle.kind;

    if speed > 1.0 && dot.abs() < 0.9 {
        let coeff = speed.max(1.0).min(9.0) / 9.0;
        kin.acceleration -= kin.velocity / coeff;
        return;
    }

    let pos = trans.position();

    let danger_length = (speed * speed / (2.0 * kind.deceleration())).min(40.0);

    let neighbors = coworld.query_around(pos, 12.0 + danger_length);

    let objs = neighbors.map(|obj| (obj.pos, coworld.get_obj(obj.id)));

    calc_decision(vehicle, map, speed, time, trans, objs);

    let speed = speed
        + ((vehicle.desired_speed - speed)
            .min(time.delta * kind.acceleration())
            .max(-time.delta * kind.deceleration()));

    let max_ang_vel = (speed.abs() / kind.min_turning_radius()).min(2.0);

    let delta_ang = direction.angle(vehicle.desired_dir);
    let mut ang = Vector2::unit_x().angle(direction);

    vehicle.ang_velocity += time.delta * kind.ang_acc();
    vehicle.ang_velocity = vehicle
        .ang_velocity
        .min(max_ang_vel)
        .min(3.0 * delta_ang.0.abs());

    ang.0 += delta_ang
        .0
        .min(vehicle.ang_velocity * time.delta)
        .max(-vehicle.ang_velocity * time.delta);
    let direction = vec2(ang.cos(), ang.sin());
    trans.set_direction(direction);
    kin.velocity = direction * speed;
}

pub fn objective_update(
    vehicle: &mut VehicleComponent,
    time: &TimeInfo,
    trans: &Transform,
    map: &Map,
) {
    match vehicle.pos_objective.last() {
        Some(p) => {
            if p.distance2(trans.position()) < OBJECTIVE_OK_DIST * OBJECTIVE_OK_DIST {
                match vehicle.objective {
                    VehicleObjective::Temporary(x) if vehicle.pos_objective.n_points() == 1 => {
                        if x.can_pass(time.time_seconds, map.lanes()) {
                            vehicle.pos_objective.pop();
                        }
                    }
                    _ => {
                        vehicle.pos_objective.pop();
                    }
                }
            }
        }
        None => match vehicle.objective {
            VehicleObjective::None => {
                let lane = map.closest_lane(trans.position());
                if let Some(id) = lane {
                    vehicle.set_travers_objective(Traversable::Lane(id), map);
                }
            }
            VehicleObjective::Temporary(x) => match x {
                Traversable::Turn(id) => {
                    vehicle.set_travers_objective(Traversable::Lane(id.dst), map);
                }
                Traversable::Lane(id) => {
                    let lane = &map.lanes()[id];

                    let neighs: Vec<(&TurnID, &Turn)> = map.intersections()[lane.dst]
                        .turns
                        .iter()
                        .filter(|(_, x)| x.id.src == id)
                        .collect();

                    if neighs.is_empty() {
                        return;
                    }

                    let r = rand::random::<f32>() * (neighs.len() as f32);
                    let (turn_id, _) = neighs[r as usize];

                    vehicle.set_travers_objective(Traversable::Turn(*turn_id), map);
                }
            },
        },
    }
}

pub fn calc_decision<'a>(
    vehicle: &'a mut VehicleComponent,
    map: &'a Map,
    speed: f32,
    time: &'a TimeInfo,
    trans: &Transform,
    neighs: impl Iterator<Item = (Vector2<f32>, &'a PhysicsObject)>,
) {
    if vehicle.wait_time > 0.0 {
        vehicle.wait_time -= time.delta;
        return;
    }
    let objective: Vector2<f32> = *match vehicle.pos_objective.last() {
        Some(x) => x,
        None => {
            return;
        }
    };

    let is_terminal = match &vehicle.objective {
        VehicleObjective::None => return,
        VehicleObjective::Temporary(_) => false,
    };

    let position = trans.position();
    let direction = trans.direction();

    let delta_pos = objective - position;
    let dist_to_pos = delta_pos.magnitude();
    let dir_to_pos: Vector2<f32> = delta_pos / dist_to_pos;
    let time_to_stop = speed / vehicle.kind.deceleration();
    let stop_dist = time_to_stop * speed / 2.0;

    let mut min_front_dist: f32 = 50.0;

    let my_ray = Ray {
        from: position - direction * vehicle.kind.width() / 2.0,
        dir: direction,
    };

    // Collision avoidance
    for nei in neighs {
        if nei.0 == position {
            continue;
        }

        let his_pos = nei.0;

        let towards_vec = his_pos - position;

        let dist2 = towards_vec.magnitude2();

        let nei_physics_obj = nei.1;

        let dist = dist2.sqrt();
        let towards_dir = towards_vec / dist;

        let dir_dot = towards_dir.dot(direction);
        let his_direction = nei_physics_obj.dir;

        // let pos_dot = towards_vec.dot(dir_normal_right);

        // front cone
        if dir_dot > 0.7 && his_direction.dot(direction) > 0.0 {
            min_front_dist =
                min_front_dist.min(dist - vehicle.kind.width() / 2.0 - nei.1.kind.width() / 2.0);
            continue;
        }

        if dir_dot < 0.0 {
            continue;
        }

        // closest win

        let his_ray = Ray {
            from: his_pos - nei.1.kind.width() / 2.0 * his_direction,
            dir: his_direction,
        };

        let inter = both_dist_to_inter(my_ray, his_ray);

        match inter {
            Some((my_dist, his_dist)) => {
                if my_dist - speed.min(2.5) < his_dist - nei_physics_obj.speed.min(2.5) {
                    continue;
                }
            }
            None => continue,
        }
        min_front_dist = min_front_dist.min(dist - vehicle.kind.width() / 2.0);
    }

    if speed.abs() < 0.2 && min_front_dist < 1.5 {
        vehicle.wait_time = rand::random::<f32>() * 0.5;
        return;
    }

    vehicle.desired_dir = dir_to_pos;
    vehicle.desired_speed = vehicle.kind.cruising_speed();

    if vehicle.pos_objective.n_points() == 1 {
        if let Temporary(trans) = vehicle.objective {
            if let Traversable::Lane(l_id) = trans {
                match map.lanes()[l_id].control.get_behavior(time.time_seconds) {
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
    }

    if is_terminal && dist_to_pos < 1.0 + stop_dist {
        // Close to terminal objective
        vehicle.desired_speed = 0.0;
    }

    if dir_to_pos.dot(direction) < 0.8 {
        // Not facing the objective
        vehicle.desired_speed = vehicle.desired_speed.min(6.0);
    }

    if min_front_dist < 1.0 + stop_dist {
        // Car in front of us
        vehicle.desired_speed = 0.0;
    }
}
