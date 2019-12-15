use crate::cars::car::CarComponent;
use crate::cars::car_graph::RoadGraph;
use crate::engine::components::{Kinematics, Transform};
use crate::engine::resources::DeltaTime;
use cgmath::{Angle, InnerSpace, Vector2};
use specs::prelude::ParallelIterator;
use specs::shred::PanicHandler;
use specs::{ParJoin, Read, System, WriteStorage};

#[derive(Default)]
pub struct CarDecision;

impl<'a> System<'a> for CarDecision {
    type SystemData = (
        Read<'a, RoadGraph, PanicHandler>,
        Read<'a, DeltaTime>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        WriteStorage<'a, CarComponent>,
    );

    fn run(
        &mut self,
        (_road_graph, delta, mut transforms, mut kinematics, mut cars): Self::SystemData,
    ) {
        let delta = delta.0;
        (&mut transforms, &mut kinematics, &mut cars)
            .par_join()
            .for_each(|(trans, kin, car)| {
                let (speed_acc, ang_acc) = car.calc_decision(trans.get_position());

                let mut speed = kin.velocity.magnitude();
                speed += speed_acc * delta;

                let mut ang = -car.direction.angle(Vector2::unit_x());
                ang.0 += ang_acc * delta;

                if kin.velocity.magnitude2() < 100.
                    || kin.velocity.normalize().dot(car.direction) > 0.9
                {
                    car.direction = Vector2::new(ang.cos(), ang.sin());
                    trans.set_angle_cos_sin(car.direction.x, car.direction.y);

                    kin.velocity = car.direction * speed;
                }
            });
    }
}
