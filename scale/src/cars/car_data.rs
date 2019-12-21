use engine::cgmath::{InnerSpace, MetricSpace, Vector2};
use engine::specs::{Builder, Component, DenseVecStorage, World, WorldExt};

use crate::cars::car_data::CarObjective::Temporary;
use engine::add_shape;
use engine::cgmath::num_traits::zero;
use engine::components::{
    CircleRender, Drag, Kinematics, MeshRenderComponent, Movable, RectRender, Transform,
};
use engine::nalgebra as na;
use engine::ncollide2d::shape::Cuboid;
use engine::RED;

#[derive(Debug)]
pub enum CarObjective {
    None,
    Temporary(Vector2<f32>),
    Terminal(Vector2<f32>),
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
        position: Vector2<f32>,
        neighs: Vec<&na::Isometry2<f32>>,
    ) -> (f32, Vector2<f32>) {
        let objective: Vector2<f32>;
        let is_terminal: bool;

        match self.objective {
            CarObjective::None => return (zero(), self.direction),
            CarObjective::Temporary(x) => {
                objective = x;
                is_terminal = false;
            }
            CarObjective::Terminal(x) => {
                objective = x;
                is_terminal = true;
            }
        }

        let mut min_dist2: f32 = 50.0 * 50.0;

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
                color: RED,
                ..Default::default()
            },
        )))
        .with(Transform::new(position))
        .with(Kinematics::from_mass(1000.0))
        .with(CarComponent {
            direction: Vector2::new(1.0, 0.0),
            objective: Temporary(objective),
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
