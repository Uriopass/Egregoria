use crate::engine_interaction::TimeInfo;
use crate::geometry::intersections::{both_dist_to_inter, Ray};
use crate::map_model::{Map, TrafficBehavior, Traversable, Turn, TurnID};
use crate::physics::{CollisionWorld, PhysicsObject};
use crate::physics::{Kinematics, Transform};
use crate::transportation::TransportComponent;
use crate::transportation::TransportObjective;
use crate::transportation::TransportObjective::Temporary;
use cgmath::{vec2, Angle, InnerSpace, MetricSpace, Vector2};
use specs::prelude::*;
use specs::shred::PanicHandler;

#[derive(Default)]
pub struct TransportDecision;

pub const OBJECTIVE_OK_DIST: f32 = 4.0;

#[derive(SystemData)]
pub struct TransportDecisionSystemData<'a> {
    map: Read<'a, Map, PanicHandler>,
    time: Read<'a, TimeInfo>,
    coworld: Read<'a, CollisionWorld, PanicHandler>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    transports: WriteStorage<'a, TransportComponent>,
}

impl<'a> System<'a> for TransportDecision {
    type SystemData = TransportDecisionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let cow = data.coworld;
        let map = &*data.map;
        let time = data.time;

        (
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.transports,
        )
            .join()
            .for_each(|(trans, kin, transport)| {
                objective_update(transport, &time, trans, &map);
                transport_physics(&cow, &map, &time, trans, kin, transport);
            });
    }
}

fn transport_physics(
    coworld: &CollisionWorld,
    map: &Map,
    time: &TimeInfo,
    trans: &mut Transform,
    kin: &mut Kinematics,
    transport: &mut TransportComponent,
) {
    let direction = trans.direction();
    let speed: f32 = kin.velocity.magnitude() * kin.velocity.dot(direction).signum();
    let dot = (kin.velocity / speed).dot(direction);
    let kind = transport.kind;

    if speed > 1.0 && dot.abs() < 0.9 {
        let coeff = speed.max(1.0).min(9.0) / 9.0;
        kin.acceleration -= kin.velocity / coeff;
        return;
    }

    let pos = trans.position();

    let danger_length = (speed * speed / (2.0 * kind.deceleration())).min(40.0);

    let neighbors = coworld.query_around(pos, 12.0 + danger_length);

    let objs = neighbors.map(|obj| (obj.pos, coworld.get_obj(obj.id)));

    calc_decision(transport, map, speed, time, trans, objs);

    let speed = speed
        + ((transport.desired_speed - speed)
            .min(time.delta * kind.acceleration())
            .max(-time.delta * kind.deceleration()));

    let max_ang_vel = (speed.abs() / kind.min_turning_radius()).min(2.0);

    let delta_ang = direction.angle(transport.desired_dir);
    let mut ang = Vector2::unit_x().angle(direction);

    transport.ang_velocity += time.delta * kind.ang_acc();
    transport.ang_velocity = transport
        .ang_velocity
        .min(max_ang_vel)
        .min(3.0 * delta_ang.0.abs());

    ang.0 += delta_ang
        .0
        .min(transport.ang_velocity * time.delta)
        .max(-transport.ang_velocity * time.delta);
    let direction = vec2(ang.cos(), ang.sin());
    trans.set_direction(direction);
    kin.velocity = direction * speed;
}

pub fn objective_update(
    transport: &mut TransportComponent,
    time: &TimeInfo,
    trans: &Transform,
    map: &Map,
) {
    match transport.pos_objective.last() {
        Some(p) => {
            if p.distance2(trans.position()) < OBJECTIVE_OK_DIST * OBJECTIVE_OK_DIST {
                match transport.objective {
                    TransportObjective::Temporary(x) if transport.pos_objective.len() == 1 => {
                        if x.can_pass(time.time_seconds, map.lanes()) {
                            transport.pos_objective.pop();
                        }
                    }
                    _ => {
                        transport.pos_objective.pop();
                    }
                }
            }
        }
        None => match transport.objective {
            TransportObjective::None => {
                let lane = map.closest_lane(trans.position());
                if let Some(id) = lane {
                    transport.set_travers_objective(Traversable::Lane(id), map);
                }
            }
            TransportObjective::Temporary(x) => match x {
                Traversable::Turn(id) => {
                    transport.set_travers_objective(Traversable::Lane(id.dst), map);
                }
                Traversable::Lane(id) => {
                    let lane = &map.lanes()[id];

                    let neighs = map.intersections()[lane.dst]
                        .turns
                        .iter()
                        .filter(|(_, x)| x.id.src == id)
                        .collect::<Vec<(&TurnID, &Turn)>>();

                    if neighs.is_empty() {
                        return;
                    }

                    let r = rand::random::<f32>() * (neighs.len() as f32);
                    let (turn_id, _) = neighs[r as usize];

                    transport.set_travers_objective(Traversable::Turn(*turn_id), map);
                }
            },
        },
    }
}

pub fn calc_decision<'a>(
    transport: &'a mut TransportComponent,
    map: &'a Map,
    speed: f32,
    time: &'a TimeInfo,
    trans: &Transform,
    neighs: impl Iterator<Item = (Vector2<f32>, &'a PhysicsObject)>,
) {
    if transport.wait_time > 0.0 {
        transport.wait_time -= time.delta;
        return;
    }
    let objective: Vector2<f32> = *match transport.pos_objective.last() {
        Some(x) => x,
        None => {
            return;
        }
    };

    let is_terminal = match &transport.objective {
        TransportObjective::None => return,
        TransportObjective::Temporary(_) => false,
    };

    let position = trans.position();
    let direction = trans.direction();

    let delta_pos = objective - position;
    let dist_to_pos = delta_pos.magnitude();
    let dir_to_pos: Vector2<f32> = delta_pos / dist_to_pos;
    let time_to_stop = speed / transport.kind.deceleration();
    let stop_dist = time_to_stop * speed / 2.0;

    let mut min_front_dist: f32 = 50.0;

    let my_ray = Ray {
        from: position - direction * transport.kind.width() / 2.0,
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
                min_front_dist.min(dist - transport.kind.width() / 2.0 - nei.1.kind.width() / 2.0);
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
        min_front_dist = min_front_dist.min(dist - transport.kind.width() / 2.0);
    }

    if speed.abs() < 0.2 && min_front_dist < 1.5 {
        transport.wait_time = rand::random::<f32>() * 0.5;
        return;
    }

    transport.desired_dir = dir_to_pos;
    transport.desired_speed = transport.kind.cruising_speed();

    if transport.pos_objective.len() == 1 {
        if let Temporary(trans) = transport.objective {
            if let Traversable::Lane(l_id) = trans {
                match map.lanes()[l_id].control.get_behavior(time.time_seconds) {
                    TrafficBehavior::RED | TrafficBehavior::ORANGE => {
                        if dist_to_pos
                            < OBJECTIVE_OK_DIST * 1.05
                                + stop_dist
                                + (transport.kind.width() / 2.0 - OBJECTIVE_OK_DIST).max(0.0)
                        {
                            transport.desired_speed = 0.0;
                        }
                    }
                    TrafficBehavior::STOP => {
                        if dist_to_pos < OBJECTIVE_OK_DIST * 0.95 + stop_dist {
                            transport.desired_speed = 0.0;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if is_terminal && dist_to_pos < 1.0 + stop_dist {
        // Close to terminal objective
        transport.desired_speed = 0.0;
    }

    if dir_to_pos.dot(direction) < 0.8 {
        // Not facing the objective
        transport.desired_speed = transport.desired_speed.min(6.0);
    }

    if min_front_dist < 1.0 + stop_dist {
        // Car in front of us
        transport.desired_speed = 0.0;
    }
}
