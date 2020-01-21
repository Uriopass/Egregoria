use crate::cars::car_data::CarObjective::{Route, Simple, Temporary};
use crate::cars::car_data::{CarComponent, CarObjective};
use crate::cars::roads::RoadGraph;
use crate::engine_interaction::TimeInfo;
use crate::physics::physics_components::{Kinematics, Transform};
use crate::physics::PhysicsWorld;
use cgmath::MetricSpace;
use cgmath::{Angle, InnerSpace, Vector2};
use nalgebra::{Isometry2, Point2};
use ncollide2d::bounding_volume::AABB;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::ParallelIterator;
use specs::shred::PanicHandler;
use specs::{ParJoin, Read, System, WriteStorage};

#[derive(Default)]
pub struct CarDecision;

const CAR_ACCELERATION: f32 = 3.0;
const CAR_DECELERATION: f32 = 3.0;
const MIN_TURNING_RADIUS: f32 = 8.0;

impl<'a> System<'a> for CarDecision {
    type SystemData = (
        Read<'a, RoadGraph, PanicHandler>,
        Read<'a, TimeInfo>,
        Read<'a, PhysicsWorld, PanicHandler>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, Kinematics>,
        WriteStorage<'a, CarComponent>,
    );

    fn run(
        &mut self,
        (road_graph, time, coworld, mut transforms, mut kinematics, mut cars): Self::SystemData,
    ) {
        let delta = time.delta;

        (&mut transforms, &mut kinematics, &mut cars)
            .par_join()
            .for_each(|(trans, kin, car)| {
                car_objective_update(car, trans, &road_graph);
                car_physics(&coworld, &road_graph, delta, trans, kin, car);
            });
    }
}

fn car_objective_update(car: &mut CarComponent, trans: &Transform, graph: &RoadGraph) {
    match car.objective {
        CarObjective::None | Simple(_) | Route(_) => {
            car.objective = Temporary(graph.closest_node(trans.position()));
        }
        CarObjective::Temporary(x) => {
            if let Some(p) = graph.nodes().get(&x).map(|x| x.pos) {
                if p.distance2(trans.position()) < 25.0 {
                    let neighs = graph.nodes().get_neighs(&x);
                    let r = rand::random::<f32>() * (neighs.len() as f32);
                    if neighs.len() == 0 {
                        return;
                    }
                    let new_obj = &neighs[r as usize].to;
                    car.objective = Temporary(*new_obj);
                }
            } else {
                car.objective = CarObjective::None;
            }
        }
    }
}

fn car_physics(
    coworld: &PhysicsWorld,
    rg: &RoadGraph,
    delta: f32,
    trans: &mut Transform,
    kin: &mut Kinematics,
    car: &mut CarComponent,
) {
    let speed: f32 = kin.velocity.magnitude();
    let dot = (kin.velocity / speed).dot(car.direction);

    if speed > 1.0 && dot < 0.9 {
        let coeff = speed.max(1.0).min(9.0) / 9.0;
        kin.acceleration -= kin.velocity / coeff;
        return;
    }

    let pos = trans.position();

    let around = AABB::new(
        Point2::new(pos.x - 50.0, pos.y - 50.0),
        Point2::new(pos.x + 50.0, pos.y + 50.0),
    );

    let all = CollisionGroups::new();

    let neighbors = coworld.interferences_with_aabb(&around, &all);

    let objs: Vec<&Isometry2<f32>> = neighbors.map(|(_, y)| y.position()).collect();

    let (desired_speed, desired_direction) = car.calc_decision(rg, speed, pos, objs);

    let speed = speed
        + ((desired_speed - speed)
            .min(delta * CAR_ACCELERATION)
            .max(-delta * CAR_DECELERATION * speed.max(3.0)));

    let ang_acc = (speed / MIN_TURNING_RADIUS).min(1.0);

    let delta_ang = car.direction.angle(desired_direction);
    let mut ang = Vector2::unit_x().angle(car.direction);

    ang.0 += delta_ang.0.min(ang_acc * delta).max(-ang_acc * delta);
    car.direction = Vector2::new(ang.cos(), ang.sin());
    trans.set_angle_cos_sin(car.direction.x, car.direction.y);

    kin.velocity = car.direction * speed;
}
