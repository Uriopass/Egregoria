use crate::cars::car::CarComponent;
use crate::cars::car_graph::RoadGraph;
use crate::engine::components::{CircleRender, Kinematics, MeshRenderComponent, Transform};
use crate::engine::resources::DeltaTime;
use crate::PhysicsWorld;
use cgmath::{Angle, InnerSpace, Vector2};
use nalgebra as na;
use ncollide2d::bounding_volume::AABB;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::ParallelIterator;
use specs::shred::PanicHandler;
use specs::world::EntitiesRes;
use specs::{Builder, Join, LazyUpdate, ParJoin, Read, System, WriteStorage};

#[derive(Default)]
pub struct CarDecision;

const CAR_ACCELERATION: f32 = 3.0;
const CAR_DECELERATION: f32 = 1.0;

impl<'a> System<'a> for CarDecision {
    type SystemData = (
        Read<'a, RoadGraph, PanicHandler>,
        Read<'a, DeltaTime>,
        Read<'a, PhysicsWorld, PanicHandler>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        WriteStorage<'a, CarComponent>,
    );

    fn run(
        &mut self,
        (_road_graph, delta, coworld, mut transforms, mut kinematics, mut cars): Self::SystemData,
    ) {
        let delta = delta.0;
        let all = CollisionGroups::new();
        (&mut transforms, &mut kinematics, &mut cars)
            .par_join()
            .for_each(|(trans, kin, car)| {
                let dot = kin.velocity.normalize().dot(car.direction);

                if kin.velocity.magnitude2() > 1.0 && dot < 0.9 {
                    let coeff = kin.velocity.magnitude().max(1.0).min(9.0) / 9.0;
                    kin.acceleration -= kin.velocity / coeff;
                    return;
                }

                let pos = trans.get_position();

                let around = AABB::new(
                    na::Point2::new(pos.x - 20.0, pos.y - 20.0),
                    na::Point2::new(pos.x + 20.0, pos.y + 20.0),
                );

                let neighbors = coworld.interferences_with_aabb(&around, &all);

                let objs: Vec<&na::Isometry2<f32>> = neighbors.map(|(_, y)| y.position()).collect();

                let (desired_speed, desired_direction) = car.calc_decision(pos, objs);

                let mut speed = kin.velocity.magnitude();
                speed += (desired_speed - speed)
                    .min(delta * CAR_ACCELERATION)
                    .max(-delta * CAR_DECELERATION * speed.max(3.0));

                let ang_acc = (speed * 0.1).min(1.0);

                let delta_ang = car.direction.angle(desired_direction);
                let mut ang = Vector2::unit_x().angle(car.direction);

                ang.0 += delta_ang.0.min(ang_acc * delta).max(-ang_acc * delta);
                car.direction = Vector2::new(ang.cos(), ang.sin());
                trans.set_angle_cos_sin(car.direction.x, car.direction.y);

                kin.velocity = car.direction * speed;
            });
    }
}
