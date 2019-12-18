use cgmath::{InnerSpace, MetricSpace, Vector2};
use specs::{Builder, Component, DenseVecStorage, World, WorldExt};

use crate::add_shape;
use crate::engine::components::{
    CircleRender, Drag, Kinematics, MeshRenderComponent, Movable, RectRender, Transform,
};
use cgmath::num_traits::zero;
use ggez::graphics::{Color, BLACK};
use nalgebra as na;
use ncollide2d::shape::Cuboid;
use std::ops::Sub;

#[derive(Component, Debug)]
pub struct CarComponent {
    pub direction: Vector2<f32>,
    pub objective: Option<Vector2<f32>>,
}

#[allow(dead_code)]
impl CarComponent {
    pub fn new(angle: f32) -> CarComponent {
        CarComponent {
            direction: Vector2::new(angle.cos(), angle.sin()),
            objective: None,
        }
    }

    pub fn normal(&self) -> Vector2<f32> {
        Vector2::new(-self.direction.y, self.direction.x)
    }

    pub fn calc_decision(
        &self,
        position: Vector2<f32>,
        neighs: Vec<&na::Isometry2<f32>>,
    ) -> (f32, Vector2<f32>) {
        if self.objective.is_none() {
            return (zero(), self.direction);
        }

        let mut max_speed2: f32 = 50.0 * 50.0;
        // Collision avoidance
        for x in neighs {
            let e_pos = Vector2::new(x.translation.x, x.translation.y);

            let dist2 = e_pos.distance2(position);
            if dist2 <= 0.0 || dist2 >= 15.0 * 15.0 {
                continue;
            }

            let e_diff = e_pos - position;
            if e_diff.normalize().dot(self.direction) < 0.75 {
                continue;
            }

            let e_direction = Vector2::new(x.rotation.re, x.rotation.im);
            if e_direction.dot(self.direction) > 0.0 {
                max_speed2 = max_speed2.min(e_diff.magnitude2());
            }
        }
        let objective = self.objective.unwrap();
        let delta_pos: Vector2<f32> = objective - position;
        ((max_speed2.sqrt() - 8.0).max(0.0), delta_pos.normalize())
    }
}

pub fn make_car_entity(world: &mut World, position: Vector2<f32>, objective: Vector2<f32>) {
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
                color: Color { r: 1.0, ..BLACK },
                ..Default::default()
            },
        )))
        .with(Transform::new(position))
        .with(Kinematics::from_mass(1000.0))
        .with(CarComponent {
            direction: Vector2::new(1.0, 0.0),
            objective: Some(objective),
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

/* ------------ old algorithm translated from java -------------------

        let objective = self.objective.unwrap();
        let delta_pos: Vector2<f32> = objective -transform;
        let angle_col = self.direction.dot(delta_pos.normalize());

        let mut angle: f64 = diff_to_target.angle(Vector2::unit_x());
        if Math::abs(angle - orientation) > Math::abs(Math::PI * 2.0 + angle - orientation) {
            angle = angle + Math::PI * 2.0;
        }

        if Math::abs(angle - orientation) > Math::abs(angle - (Math::PI * 2.0 + orientation)) {
            orientation = orientation + Math::PI * 2;
        }


        if speed > 1 {
            let actual_turn_speed: f64 = turn_speed * (Math::min(speed, 10) / 10);
            orientation += Math::signum(angle - orientation) * Math::min(&Math::abs(angle - orientation), actual_turn_speed * delta);
            self.orientationVec = Vector2<f32>::new(&Math::cos(orientation), &Math::sin(orientation));
        }

        let mut desired_speed: f32;
        let dist_to_target = delta_pos.magnitude();
        if dist_to_target < 1.0 {
            // . || (dist_to_target < 25.0 && (get_current_target().state(time) >= 1)) {
            desired_speed = 20.0
        } else {
            if angle_col > (f32::PI() / 8.0).cos() {
                desired_speed = 60.0;
            } else {
                desired_speed = f32::min(30.0, delta_pos.magnitude() as f32 / 2.0);
            }

            //System.out.println("-------");
            for (enemy_pos, enemy) in neighbors {
                if enemy_pos ==transform {
                    continue;
                }

                let dist2: f32 = enemy_pos.distance2(transform);
                let dist_check = 20.0 + speed / 2.0;
                if dist2 < dist_check * dist_check {
                    let dot: f32 = enemy.direction.dot(self.direction);
                    if dot > 0.0 {
                        let cos0: f32 = delta_pos.dot(self.direction) / (dist_to_target);
                        if cos0 > 0.8 || (cos0 > 0.0 && (dist2 * (1.0 - cos0) < 3.0 * 3.0)) {
                            desired_speed = 0.0;
                        }
                    }
                }
            }
        }
        let acc: f64 = if desired_speed > speed { 10.0 } else { 3.0 * 10.0 };
        // something something bam bam acceleration and angular acceleration
*/
