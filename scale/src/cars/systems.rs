use crate::cars::data::CarObjective::{Route, Simple, Temporary};
use crate::cars::data::{CarComponent, CarObjective};
use crate::engine_interaction::TimeInfo;
use crate::map::RoadGraph;
use crate::physics::PhysicsWorld;
use crate::physics::{Kinematics, Transform};
use cgmath::MetricSpace;
use cgmath::{Angle, InnerSpace, Vector2};
use nalgebra::{Isometry2, Point2};
use ncollide2d::bounding_volume::AABB;
use ncollide2d::pipeline::CollisionGroups;
use specs::prelude::*;
use specs::shred::PanicHandler;

#[derive(Default)]
pub struct CarDecision;

pub const CAR_ACCELERATION: f32 = 3.0;
pub const CAR_DECELERATION: f32 = 9.0;
pub const MIN_TURNING_RADIUS: f32 = 6.0;

#[derive(SystemData)]
pub struct CarDecisionSystemData<'a> {
    rg: Read<'a, RoadGraph, PanicHandler>,
    time: Read<'a, TimeInfo>,
    coworld: Read<'a, PhysicsWorld, PanicHandler>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    cars: WriteStorage<'a, CarComponent>,
}

impl<'a> System<'a> for CarDecision {
    type SystemData = CarDecisionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let cow = data.coworld;
        let rg = data.rg;
        let time = data.time;

        let x = std::time::Instant::now();

        (&mut data.transforms, &mut data.kinematics, &mut data.cars)
            .par_join()
            .for_each(|(trans, kin, car)| {
                car_objective_update(car, &time, trans, &rg);
                car_physics(&cow, &rg, &time, trans, kin, car);
            });

        println!(
            "Updating cars took {}",
            (std::time::Instant::now() - x).as_secs_f32() * 1000.0
        );
    }
}

fn car_objective_update(
    car: &mut CarComponent,
    time: &TimeInfo,
    trans: &Transform,
    graph: &RoadGraph,
) {
    match car.objective {
        CarObjective::None | Simple(_) | Route(_) => {
            car.objective = graph
                .closest_node(trans.position())
                .map_or(CarObjective::None, Temporary);
        }
        CarObjective::Temporary(x) => {
            if let Some(p) = graph.nodes().get(x).map(|x| x.pos) {
                if p.distance2(trans.position()) < 25.0
                    && !graph.nodes()[&x]
                        .light
                        .get_color(time.time_seconds)
                        .is_red()
                {
                    let neighs = graph.nodes().get_neighs(x);
                    let r = rand::random::<f32>() * (neighs.len() as f32);
                    if neighs.is_empty() {
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
    time: &TimeInfo,
    trans: &mut Transform,
    kin: &mut Kinematics,
    car: &mut CarComponent,
) {
    let speed: f32 = kin.velocity.magnitude() * kin.velocity.dot(car.direction).signum();
    let dot = (kin.velocity / speed).dot(car.direction);

    if speed > 1.0 && dot.abs() < 0.9 {
        let coeff = speed.max(1.0).min(9.0) / 9.0;
        kin.acceleration -= kin.velocity / coeff;
        return;
    }

    let pos = trans.position();

    let danger_length = (speed * speed / (2.0 * CAR_DECELERATION)).max(10.0);

    let around = AABB::new(
        Point2::new(pos.x - danger_length, pos.y - danger_length),
        Point2::new(pos.x + danger_length, pos.y + danger_length),
    );

    let all = CollisionGroups::new();

    let neighbors = coworld.interferences_with_aabb(&around, &all);

    let objs: Vec<&Isometry2<f32>> = neighbors.map(|(_, y)| y.position()).collect();

    car.calc_decision(rg, speed, time, pos, objs);

    let speed = speed
        + ((car.desired_speed - speed)
            .min(time.delta * CAR_ACCELERATION)
            .max(-time.delta * CAR_DECELERATION));

    let ang_acc = (speed.abs() / MIN_TURNING_RADIUS).min(2.0);

    let delta_ang = car.direction.angle(car.desired_dir);
    let mut ang = Vector2::unit_x().angle(car.direction);

    ang.0 += delta_ang
        .0
        .min(ang_acc * time.delta)
        .max(-ang_acc * time.delta);
    car.direction = Vector2::new(ang.cos(), ang.sin());
    trans.set_angle_cos_sin(car.direction.x, car.direction.y);

    kin.velocity = car.direction * speed;
}
