use crate::cars::data::CarObjective::{Route, Simple, Temporary};
use crate::cars::data::{CarComponent, CarObjective};
use crate::engine_interaction::TimeInfo;
use crate::map_model::{Map, NavMesh};
use crate::physics::PhysicsWorld;
use crate::physics::{Kinematics, Transform};
use cgmath::MetricSpace;
use cgmath::{Angle, InnerSpace, Vector2};
use specs::prelude::*;
use specs::shred::PanicHandler;

#[derive(Default)]
pub struct CarDecision;

pub const CAR_ACCELERATION: f32 = 3.0;
pub const CAR_DECELERATION: f32 = 9.0;
pub const MIN_TURNING_RADIUS: f32 = 6.0;

#[derive(SystemData)]
pub struct CarDecisionSystemData<'a> {
    map: Read<'a, Map, PanicHandler>,
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
        let navmesh = &data.map.navmesh;
        let time = data.time;

        (&mut data.transforms, &mut data.kinematics, &mut data.cars)
            .par_join()
            .for_each(|(trans, kin, car)| {
                car_objective_update(car, &time, trans, &navmesh);
                car_physics(&cow, &navmesh, &time, trans, kin, car);
            });
    }
}

fn car_objective_update(
    car: &mut CarComponent,
    time: &TimeInfo,
    trans: &Transform,
    navmesh: &NavMesh,
) {
    match car.objective {
        CarObjective::None | Simple(_) | Route(_) => {
            car.objective = navmesh
                .closest_node(trans.position())
                .map_or(CarObjective::None, Temporary);
        }
        CarObjective::Temporary(x) => {
            if let Some(p) = navmesh.get(x).map(|x| x.pos) {
                if p.distance2(trans.position()) < 25.0
                    && !navmesh[&x].light.get_color(time.time_seconds).is_red()
                {
                    let neighs = navmesh.get_neighs(x);
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
    navmesh: &NavMesh,
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

    let danger_length = (speed * speed / (2.0 * CAR_DECELERATION)).min(40.0);

    let neighbors = coworld.query_around(pos, 10.0 + danger_length);

    let objs = neighbors.map(|obj| &obj.pos);

    car.calc_decision(navmesh, speed, time, pos, objs);

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
