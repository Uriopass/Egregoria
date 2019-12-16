use crate::cars::car::CarComponent;
use crate::cars::car_graph::RoadGraph;
use crate::engine::components::{Kinematics, Transform};
use crate::engine::resources::DeltaTime;
use crate::PhysicsWorld;
use cgmath::{Angle, InnerSpace, Vector2};
use nalgebra as na;
use ncollide2d::bounding_volume::AABB;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::ParallelIterator;
use specs::shred::PanicHandler;
use specs::{ParJoin, Read, System, WriteStorage};

#[derive(Default)]
pub struct CarDecision;

const CAR_ACC: f32 = 20.0;
const CAR_DEC: f32 = 20.0;

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
                let pos = trans.get_position();
                let around = AABB::new(
                    na::Point2::new(pos.x - 100.0, pos.y - 100.0),
                    na::Point2::new(pos.x + 100.0, pos.y + 100.0),
                );

                let neighbors = coworld.interferences_with_aabb(&around, &all);

                let objs: Vec<&na::Isometry2<f32>> = neighbors.map(|(_, y)| y.position()).collect();

                let (desired_speed, desired_angle) = car.calc_decision(pos, objs);

                let mut speed = kin.velocity.magnitude();
                //speed += (desired_speed - speed).min(delta);

                let mut ang = -car.direction.angle(Vector2::unit_x());
                ang.0 += desired_angle * delta;

                if kin.velocity.magnitude2() < 100.0
                    || kin.velocity.normalize().dot(car.direction) > 0.9
                {
                    car.direction = Vector2::new(ang.cos(), ang.sin());
                    trans.set_angle_cos_sin(car.direction.x, car.direction.y);

                    kin.velocity = car.direction * speed;
                }
            });
    }
}
