use crate::engine_interaction::TimeInfo;
use crate::geometry::intersections::{both_dist_to_inter, Ray};
use crate::gui::{InspectDragf, InspectVec2, InspectVecVector};
use crate::map_model::{Map, TrafficBehavior, Traversable, Turn, TurnID};
use crate::physics::{PhysicsObject, Transform};
use crate::transportation::systems::{CAR_DECELERATION, OBJECTIVE_OK_DIST};
use crate::transportation::transport_component::TransportObjective::Temporary;
use crate::transportation::CAR_WIDTH;
use cgmath::{InnerSpace, MetricSpace, Vector2};
use imgui::{im_str, Ui};
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage, World};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportObjective {
    None,
    Temporary(Traversable),
}

impl<'a> InspectRenderDefault<TransportObjective> for TransportObjective {
    fn render(
        _: &[&TransportObjective],
        _: &'static str,
        _: &mut World,
        _: &Ui,
        _: &InspectArgsDefault,
    ) {
        unimplemented!();
    }

    fn render_mut(
        data: &mut [&mut TransportObjective],
        label: &'static str,
        _: &mut World,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            return false;
        }

        let obj = &data[0];
        match obj {
            TransportObjective::None => ui.text(im_str!("None {}", label)),
            TransportObjective::Temporary(x) => ui.text(im_str!("{:?} {}", x, label)),
        }

        false
    }
}

#[derive(Component, Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct TransportComponent {
    pub objective: TransportObjective,
    #[inspect(proxy_type = "InspectVecVector")]
    pub pos_objective: Vec<Vector2<f32>>,
    #[inspect(proxy_type = "InspectDragf")]
    pub desired_speed: f32,
    #[inspect(proxy_type = "InspectVec2")]
    pub desired_dir: Vector2<f32>,
    #[inspect(proxy_type = "InspectDragf")]
    pub ang_velocity: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub wait_time: f32,
}

impl TransportComponent {
    pub fn new(objective: TransportObjective) -> TransportComponent {
        TransportComponent {
            objective,
            desired_speed: 0.0,
            desired_dir: Vector2::<f32>::new(0.0, 0.0),
            wait_time: 0.0,
            ang_velocity: 0.0,
            pos_objective: Vec::with_capacity(7),
        }
    }

    fn set_travers_objective(&mut self, travers: Traversable, map: &Map) {
        self.objective = Temporary(travers);
        let p = travers.points(map);
        self.pos_objective.extend(p.iter().rev());
    }

    pub fn objective_update(&mut self, time: &TimeInfo, trans: &Transform, map: &Map) {
        match self.pos_objective.last() {
            Some(p) => {
                if p.distance2(trans.position()) < OBJECTIVE_OK_DIST * OBJECTIVE_OK_DIST {
                    match self.objective {
                        TransportObjective::Temporary(x) if self.pos_objective.len() == 1 => {
                            if x.can_pass(time.time_seconds, map.lanes()) {
                                self.pos_objective.pop();
                            }
                        }
                        _ => {
                            self.pos_objective.pop();
                        }
                    }
                }
            }
            None => match self.objective {
                TransportObjective::None => {
                    let lane = map.closest_lane(trans.position());
                    if let Some(id) = lane {
                        self.set_travers_objective(Traversable::Lane(id), map);
                    }
                }
                TransportObjective::Temporary(x) => match x {
                    Traversable::Turn(id) => {
                        self.set_travers_objective(Traversable::Lane(id.dst), map);
                    }
                    Traversable::Lane(id) => {
                        let lane = &map.lanes()[id];

                        let neighs = map.intersections()[lane.forward_dst_inter()]
                            .turns
                            .iter()
                            .filter(|(_, x)| x.id.src == id)
                            .collect::<Vec<(&TurnID, &Turn)>>();

                        if neighs.is_empty() {
                            return;
                        }

                        let r = rand::random::<f32>() * (neighs.len() as f32);
                        let (turn_id, _) = neighs[r as usize];

                        self.set_travers_objective(Traversable::Turn(*turn_id), map);
                    }
                },
            },
        }
    }

    pub fn calc_decision<'a>(
        &'a mut self,
        map: &'a Map,
        speed: f32,
        time: &'a TimeInfo,
        trans: &Transform,
        neighs: impl Iterator<Item = (Vector2<f32>, &'a PhysicsObject)>,
    ) {
        if self.wait_time > 0.0 {
            self.wait_time -= time.delta;
            return;
        }
        let objective: Vector2<f32> = *match self.pos_objective.last() {
            Some(x) => x,
            None => {
                return;
            }
        };

        let is_terminal = match &self.objective {
            TransportObjective::None => return,
            TransportObjective::Temporary(_) => false,
        };

        let position = trans.position();
        let direction = trans.direction();

        let delta_pos = objective - position;
        let dist_to_pos = delta_pos.magnitude();
        let dir_to_pos: Vector2<f32> = delta_pos / dist_to_pos;
        let time_to_stop = speed / CAR_DECELERATION;
        let stop_dist = time_to_stop * speed / 2.0;

        let mut min_front_dist: f32 = 50.0;

        let my_ray = Ray {
            from: position - direction * CAR_WIDTH / 2.0,
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

            if dist2 > (6.0 + stop_dist) * (6.0 + stop_dist) {
                continue;
            }

            let nei_physics_obj = nei.1;

            let dist = dist2.sqrt();
            let towards_dir = towards_vec / dist;

            let dir_dot = towards_dir.dot(direction);
            let his_direction = nei_physics_obj.dir;

            // let pos_dot = towards_vec.dot(dir_normal_right);

            // front cone
            if dir_dot > 0.7 && his_direction.dot(direction) > 0.0 {
                min_front_dist = min_front_dist.min(dist);
                continue;
            }

            if dir_dot < 0.0 {
                continue;
            }

            // closest win

            let his_ray = Ray {
                from: his_pos - CAR_WIDTH / 2.0 * his_direction,
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
            min_front_dist = min_front_dist.min(dist);
        }

        if speed.abs() < 0.2 && min_front_dist < 7.0 {
            self.wait_time = rand::random::<f32>() * 0.5;
            return;
        }

        self.desired_dir = dir_to_pos;
        self.desired_speed = 15.0;

        if self.pos_objective.len() == 1 {
            if let Temporary(trans) = self.objective {
                if let Traversable::Lane(l_id) = trans {
                    match map.lanes()[l_id].control.get_behavior(time.time_seconds) {
                        TrafficBehavior::RED | TrafficBehavior::ORANGE => {
                            if dist_to_pos < OBJECTIVE_OK_DIST * 1.05 + stop_dist {
                                self.desired_speed = 0.0;
                            }
                        }
                        TrafficBehavior::STOP => {
                            if dist_to_pos < OBJECTIVE_OK_DIST * 0.95 + stop_dist {
                                self.desired_speed = 0.0;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if is_terminal && dist_to_pos < 1.0 + stop_dist {
            // Close to terminal objective
            self.desired_speed = 0.0;
        }

        if dir_to_pos.dot(direction) < 0.8 {
            // Not facing the objective
            self.desired_speed = self.desired_speed.min(6.0);
        }

        if min_front_dist < 6.0 + stop_dist {
            // Car in front of us
            self.desired_speed = 0.0;
        }
    }
}
