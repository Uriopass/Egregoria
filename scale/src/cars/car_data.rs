use cgmath::{InnerSpace, MetricSpace, Vector2};
use engine::specs::{Builder, Component, DenseVecStorage, World, WorldExt};

use crate::cars::car_data::CarObjective::{Simple, Temporary};
use crate::cars::car_graph::RoadGraph;
use crate::graphs::graph::NodeID;
use cgmath::num_traits::zero;
use engine::add_shape;
use engine::components::{
    CircleRender, Drag, Kinematics, MeshRenderComponent, Movable, RectRender, Transform,
};
use engine::nalgebra::Isometry2;
use engine::ncollide2d::shape::Cuboid;
use engine::RED;

#[allow(dead_code)]
#[derive(Debug)]
pub enum CarObjective {
    None,
    Simple(NodeID),
    Temporary(NodeID),
    Route(Vec<NodeID>),
}

impl CarObjective {
    pub fn to_pos(&self, rg: &RoadGraph) -> Option<Vector2<f32>> {
        match self {
            CarObjective::None => None,
            Simple(x) | Temporary(x) => rg.0.nodes.get(x).map(|x| x.pos),
            CarObjective::Route(l) => l.get(0).and_then(|x| rg.0.nodes.get(x).map(|x| x.pos)),
        }
    }
}

#[derive(Component, Debug)]
pub struct CarComponent {
    pub direction: Vector2<f32>,
    pub objective: CarObjective,
}

#[allow(dead_code)]
impl CarComponent {
    pub fn new(angle: f32) -> CarComponent {
        CarComponent {
            direction: Vector2::new(angle.cos(), angle.sin()),
            objective: CarObjective::None,
        }
    }

    pub fn normal(&self) -> Vector2<f32> {
        Vector2::new(-self.direction.y, self.direction.x)
    }

    pub fn calc_decision(
        &self,
        rg: &RoadGraph,
        speed: f32,
        position: Vector2<f32>,
        neighs: Vec<&Isometry2<f32>>,
    ) -> (f32, Vector2<f32>) {
        let objective: Vector2<f32> = match self.objective.to_pos(rg) {
            Some(x) => x,
            None => {
                return (0.0, self.direction);
            }
        };

        let is_terminal = match &self.objective {
            CarObjective::None => return (zero(), self.direction),
            CarObjective::Simple(_x) => true,
            CarObjective::Temporary(_x) => false,
            CarObjective::Route(x) => x.len() == 1,
        };

        let mut min_dist2: f32 = 50.0 * 50.0;

        // Collision avoidance
        for x in neighs {
            let e_pos = Vector2::new(x.translation.x, x.translation.y);

            let dist2 = e_pos.distance2(position);
            if dist2 <= 0.0 || dist2 >= 15.0 * 15.0 + speed * speed {
                continue;
            }

            let e_diff = e_pos - position;
            if e_diff.normalize().dot(self.direction) < 0.75 {
                continue;
            }

            let e_direction = Vector2::new(x.rotation.re, x.rotation.im);
            if e_direction.dot(self.direction) > 0.0 {
                min_dist2 = min_dist2.min(e_diff.magnitude2());
            }
        }

        let delta_pos = objective - position;
        let dist_to_pos = delta_pos.magnitude();
        let dir_to_pos: Vector2<f32> = delta_pos / dist_to_pos;

        let mut speed: f32 = 50.0;

        if is_terminal {
            speed = dist_to_pos;
        }
        if dir_to_pos.dot(self.direction) < 0.8 {
            speed = 7.0;
        }
        (speed.min(min_dist2.sqrt() - 8.0).max(0.0), dir_to_pos)
    }
}

pub fn make_car_entity(world: &mut World, position: Vector2<f32>) {
    let car_width = 4.5;
    let car_height = 2.0;

    let e = world
        .create_entity()
        .with(MeshRenderComponent::from((
            RectRender {
                width: car_width,
                height: car_height,
                ..Default::default()
            },
            CircleRender {
                radius: 0.3,
                offset: Vector2::new(car_width / 2.0, 0.0),
                color: RED,
                ..Default::default()
            },
        )))
        .with(Transform::new(position))
        .with(Kinematics::from_mass(1000.0))
        .with(CarComponent {
            direction: Vector2::new(1.0, 0.0),
            objective: CarObjective::None,
        })
        .with(Drag::new(0.3))
        .with(Movable)
        .build();

    add_shape(
        world,
        e,
        Cuboid::new([car_width / 2.0, car_height / 2.0].into()),
    )
}
