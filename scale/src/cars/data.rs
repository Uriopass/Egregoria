use crate::cars::data::CarObjective::{Simple, Temporary};
use crate::cars::systems::CAR_DECELERATION;
use crate::engine_interaction::TimeInfo;
use crate::graphs::graph::NodeID;
use crate::gui::{ImCgVec2, ImDragf};
use crate::interaction::{Movable, Selectable};
use crate::map::{RoadGraph, TrafficLightColor};
use crate::physics::add_shape;
use crate::physics::{Kinematics, Transform};
use crate::rendering::meshrender_component::{CircleRender, MeshRender, RectRender};
use crate::rendering::RED;
use cgmath::{InnerSpace, Vector2};
use imgui::{im_str, Ui};
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::*;
use nalgebra::Isometry2;
use ncollide2d::shape::Cuboid;
use serde::{Deserialize, Serialize};
use specs::{Builder, Component, DenseVecStorage, Entity, World, WorldExt};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CarObjective {
    None,
    Simple(NodeID),
    Temporary(NodeID),
    Route(Vec<NodeID>),
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
        let pos: Option<Vector2<f32>>;
        {
            let rg = world.read_resource::<RoadGraph>();
            pos = data[0].to_pos(&*rg);
        }
        match pos {
            Some(x) => {
                <ImCgVec2 as InspectRenderDefault<Vector2<f32>>>::render(
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
    pub fn to_pos(&self, rg: &RoadGraph) -> Option<Vector2<f32>> {
        match self {
            CarObjective::None => None,
            Simple(x) | Temporary(x) => rg.nodes().get(*x).map(|x| x.pos),
            CarObjective::Route(l) => l.get(0).and_then(|x| rg.nodes().get(*x).map(|x| x.pos)),
        }
    }
}

#[derive(Component, Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct CarComponent {
    #[inspect(proxy_type = "ImCgVec2")]
    pub direction: Vector2<f32>,
    pub objective: CarObjective,
    #[inspect(proxy_type = "ImDragf")]
    pub desired_speed: f32,
    #[inspect(proxy_type = "ImCgVec2")]
    pub desired_dir: Vector2<f32>,
    #[inspect(proxy_type = "ImDragf")]
    pub wait_time: f32,
}

#[allow(dead_code)]
impl CarComponent {
    pub fn new(direction: Vector2<f32>) -> CarComponent {
        CarComponent {
            direction,
            objective: CarObjective::None,
            desired_speed: 0.0,
            desired_dir: Vector2::<f32>::new(0.0, 0.0),
            wait_time: 0.0,
        }
    }

    pub fn normal(&self) -> Vector2<f32> {
        Vector2::new(-self.direction.y, self.direction.x)
    }

    pub fn calc_decision(
        &mut self,
        rg: &RoadGraph,
        speed: f32,
        time: &TimeInfo,
        position: Vector2<f32>,
        neighs: Vec<&Isometry2<f32>>,
    ) {
        if self.wait_time > 0.0 {
            self.wait_time -= time.delta;
            return;
        }
        let objective: Vector2<f32> = match self.objective.to_pos(rg) {
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

        let mut min_front_dist: f32 = 50.0;

        // Collision avoidance
        for x in neighs {
            let e_pos = Vector2::new(x.translation.x, x.translation.y);

            let e_diff = e_pos - position;
            let e_dist = e_diff.magnitude();
            if e_dist < 1e-5 {
                // dont check self
                continue;
            }

            if (e_diff / e_dist).dot(self.direction) < 0.75 {
                continue;
            }

            let same_direction =
                Vector2::new(x.rotation.re, x.rotation.im).dot(self.direction) > 0.0; // Avoid traffic jams by only considering same direction cars

            if same_direction {
                min_front_dist = min_front_dist.min(e_dist);
            }
        }

        if speed.abs() < 0.2 && min_front_dist < 7.0 {
            self.wait_time = rand::random::<f32>() * 0.5;
            return;
        }

        let delta_pos = objective - position;
        let dist_to_pos = delta_pos.magnitude();
        let dir_to_pos: Vector2<f32> = delta_pos / dist_to_pos;
        self.desired_dir = dir_to_pos;
        self.desired_speed = 15.0;
        let time_to_stop = speed / CAR_DECELERATION;
        let stop_dist = time_to_stop * speed / 2.0;

        if let Temporary(n_id) = self.objective {
            match rg.nodes()[&n_id].light.get_color(time.time_seconds) {
                TrafficLightColor::RED => {
                    if dist_to_pos < 5.0 + stop_dist {
                        self.desired_speed = 0.0;
                    }
                }
                TrafficLightColor::ORANGE(time_left) => {
                    if speed * time_left <= dist_to_pos  // if I don't to have the time to go through by keeping my speed
                        && dist_to_pos < 5.0 + stop_dist
                    // and I should slow down to stop
                    {
                        self.desired_speed = 0.0; // stop
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
            self.desired_speed = self.desired_speed.min(10.0);
        }

        if min_front_dist < 6.0 + stop_dist {
            // Car in front of us
            self.desired_speed = 0.0;
        }
    }
}

pub fn make_car_entity(world: &mut World, trans: Transform, car: CarComponent) -> Entity {
    let car_width = 4.5;
    let car_height = 2.0;

    let mut mr = MeshRender::empty(3);
    mr.add(RectRender {
        width: car_width,
        height: car_height,
        ..Default::default()
    })
    .add(CircleRender {
        radius: 0.3,
        offset: Vector2::new(car_width / 2.0, 0.0),
        color: RED,
        ..Default::default()
    });

    let e = world
        .create_entity()
        .with(mr)
        .with(trans)
        .with(Kinematics::from_mass(1000.0))
        .with(car)
        .with(Movable)
        .with(Selectable)
        .build();

    add_shape(
        world,
        e,
        Cuboid::new([car_width / 2.0, car_height / 2.0].into()),
    );
    e
}
