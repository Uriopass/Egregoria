use crate::cars::data::CarObjective::{Simple, Temporary};
use crate::cars::systems::{CAR_DECELERATION, OBJECTIVE_OK_DIST};
use crate::engine_interaction::TimeInfo;
use crate::geometry::intersections::{both_dist_to_inter, Ray};
use crate::gui::{InspectDragf, InspectVec2};
use crate::interaction::{Movable, Selectable};
use crate::map_model::{Map, NavMesh, NavNodeID, TrafficBehavior};
use crate::physics::{
    add_to_coworld, Collider, Kinematics, PhysicsObject, PhysicsWorld, Transform,
};
use crate::rendering::meshrender_component::{CircleRender, MeshRender, RectRender};
use crate::rendering::{Color, BLACK, GREEN};
use cgmath::{vec2, InnerSpace, Vector2};
use imgui::{im_str, Ui};
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Builder, Component, DenseVecStorage, Entity, World, WorldExt};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CarObjective {
    None,
    Simple(NavNodeID),
    Temporary(NavNodeID),
    Route(Vec<NavNodeID>),
}

impl<'a> InspectRenderDefault<CarObjective> for CarObjective {
    fn render(_: &[&CarObjective], _: &'static str, _: &mut World, _: &Ui, _: &InspectArgsDefault) {
        unimplemented!();
    }

    fn render_mut(
        data: &mut [&mut CarObjective],
        label: &'static str,
        world: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            return false;
        }

        // TODO: Handle Route
        let pos: Option<Vector2<f32>> = data[0].to_pos(&world.read_resource::<Map>().navmesh());
        match pos {
            Some(x) => {
                <InspectVec2 as InspectRenderDefault<Vector2<f32>>>::render(
                    &[&x],
                    label,
                    world,
                    ui,
                    args,
                );
            }
            None => ui.text(im_str!("No objective {}", label)),
        };
        false
    }
}

impl CarObjective {
    pub fn to_pos(&self, navmesh: &NavMesh) -> Option<Vector2<f32>> {
        match self {
            CarObjective::None => None,
            Simple(x) | Temporary(x) => navmesh.get(*x).map(|x| x.pos),
            CarObjective::Route(l) => l.get(0).and_then(|x| navmesh.get(*x).map(|x| x.pos)),
        }
    }
}

#[derive(Component, Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct CarComponent {
    #[inspect(proxy_type = "InspectVec2")]
    pub direction: Vector2<f32>,
    pub objective: CarObjective,
    #[inspect(proxy_type = "InspectDragf")]
    pub desired_speed: f32,
    #[inspect(proxy_type = "InspectVec2")]
    pub desired_dir: Vector2<f32>,
    #[inspect(proxy_type = "InspectDragf")]
    pub wait_time: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub ang_velocity: f32,
}

impl CarComponent {
    pub fn new(direction: Vector2<f32>, objective: CarObjective) -> CarComponent {
        CarComponent {
            direction,
            objective,
            desired_speed: 0.0,
            desired_dir: Vector2::<f32>::new(0.0, 0.0),
            wait_time: 0.0,
            ang_velocity: 0.0,
        }
    }

    pub fn calc_decision<'a>(
        &'a mut self,
        navmesh: &'a NavMesh,
        speed: f32,
        time: &'a TimeInfo,
        position: Vector2<f32>,
        neighs: impl Iterator<Item = (Vector2<f32>, &'a PhysicsObject)>,
    ) {
        if self.wait_time > 0.0 {
            self.wait_time -= time.delta;
            return;
        }
        let objective: Vector2<f32> = match self.objective.to_pos(navmesh) {
            Some(x) => x,
            None => {
                return;
            }
        };

        let is_terminal = match &self.objective {
            CarObjective::None => return,
            CarObjective::Simple(_) => true,
            CarObjective::Temporary(_) => false,
            CarObjective::Route(x) => x.len() == 1,
        };

        let delta_pos = objective - position;
        let dist_to_pos = delta_pos.magnitude();
        let dir_to_pos: Vector2<f32> = delta_pos / dist_to_pos;
        let time_to_stop = speed / CAR_DECELERATION;
        let stop_dist = time_to_stop * speed / 2.0;

        let mut min_front_dist: f32 = 50.0;

        let my_ray = Ray {
            from: position - self.direction * CAR_WIDTH / 2.0,
            dir: self.direction,
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

            let dir_dot = towards_dir.dot(self.direction);
            let his_direction = nei_physics_obj.dir;

            // let pos_dot = towards_vec.dot(dir_normal_right);

            // front cone
            if dir_dot > 0.7 && his_direction.dot(self.direction) > 0.0 {
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
                    if my_dist - speed.min(1.0) < his_dist - nei_physics_obj.speed.min(1.0) {
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

        if let Temporary(n_id) = self.objective {
            match navmesh[&n_id].control.get_behavior(time.time_seconds) {
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

        if is_terminal && dist_to_pos < 1.0 + stop_dist {
            // Close to terminal objective
            self.desired_speed = 0.0;
        }

        if dir_to_pos.dot(self.direction) < 0.8 {
            // Not facing the objective
            self.desired_speed = self.desired_speed.min(6.0);
        }

        if min_front_dist < 6.0 + stop_dist {
            // Car in front of us
            self.desired_speed = 0.0;
        }
    }
}

pub fn get_random_car_color() -> Color {
    let car_colors: [(Color, f32); 9] = [
        (Color::from_hex(0x22_22_22), 0.22),  // Black
        (Color::from_hex(0xff_ff_ff), 0.19),  // White
        (Color::from_hex(0x66_66_66), 0.17),  // Gray
        (Color::from_hex(0xb8_b8_b8), 0.14),  // Silver
        (Color::from_hex(0x1a_3c_70), 0.1),   // Blue
        (Color::from_hex(0xd8_22_00), 0.1),   // Red
        (Color::from_hex(0x7c_4b_24), 0.02),  // Brown
        (Color::from_hex(0xd4_c6_78), 0.015), // Gold
        (Color::from_hex(0x72_cb_19), 0.015), // Green
    ];

    let total: f32 = car_colors.iter().map(|x| x.1).sum();

    let r = rand::random::<f32>() * total;
    let mut partial = 0.0;
    for (col, freq) in &car_colors {
        partial += freq;
        if partial >= r {
            return *col;
        }
    }
    unreachable!();
}

const CAR_WIDTH: f32 = 4.5;
const CAR_HEIGHT: f32 = 2.0;

pub fn make_car_entity(world: &mut World, trans: Transform, car: CarComponent) -> Entity {
    let is_tank = false;
    let mut mr = MeshRender::empty(3);

    let c = Color::from_hex(0x25_66_29);
    if is_tank {
        mr.add(RectRender {
            width: 5.0,
            height: 3.0,
            color: GREEN,
            ..Default::default()
        })
        .add(RectRender {
            width: 4.0,
            height: 1.0,
            offset: [2.0, 0.0].into(),
            color: c,
            ..Default::default()
        })
        .add(CircleRender {
            radius: 0.5,
            offset: vec2(4.0, 0.0),
            color: c,
            ..Default::default()
        });
    } else {
        mr.add(RectRender {
            width: CAR_WIDTH,
            height: CAR_HEIGHT,
            color: get_random_car_color(),
            ..Default::default()
        })
        .add(RectRender {
            width: 0.4,
            height: 1.8,
            offset: [-1.7, 0.0].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 1.0,
            height: 1.6,
            offset: [0.8, 0.0].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 2.7,
            height: 0.15,
            offset: [-0.4, 0.85].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 2.7,
            height: 0.15,
            offset: [-0.4, -0.85].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 0.4,
            height: 0.15,
            offset: [2.1, -0.7].into(),
            color: BLACK,
            ..Default::default()
        })
        .add(RectRender {
            width: 0.4,
            height: 0.15,
            offset: [2.1, 0.7].into(),
            color: BLACK,
            ..Default::default()
        });
    }

    let e = world
        .create_entity()
        .with(mr)
        .with(trans)
        .with(Kinematics::from_mass(1000.0))
        .with(car)
        .with(Movable)
        .with(Selectable)
        .build();

    add_to_coworld(world, e);
    e
}

pub fn delete_car_entity(world: &mut World, e: Entity) {
    {
        let handle = world.read_component::<Collider>().get(e).unwrap().0;
        let mut coworld = world.write_resource::<PhysicsWorld>();
        coworld.remove(handle);
    }
    world.delete_entity(e).unwrap();
}
