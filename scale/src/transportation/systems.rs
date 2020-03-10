use crate::engine_interaction::TimeInfo;
use crate::map_model::Map;
use crate::physics::CollisionWorld;
use crate::physics::{Kinematics, Transform};
use crate::transportation::transport_component::TransportComponent;
use cgmath::{vec2, Angle, InnerSpace, Vector2};
use specs::prelude::*;
use specs::shred::PanicHandler;

#[derive(Default)]
pub struct TransportDecision;

pub const OBJECTIVE_OK_DIST: f32 = 4.0;

#[derive(SystemData)]
pub struct TransportDecisionSystemData<'a> {
    map: Read<'a, Map, PanicHandler>,
    time: Read<'a, TimeInfo>,
    coworld: Read<'a, CollisionWorld, PanicHandler>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    transports: WriteStorage<'a, TransportComponent>,
}

impl<'a> System<'a> for TransportDecision {
    type SystemData = TransportDecisionSystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let cow = data.coworld;
        let map = &*data.map;
        let time = data.time;

        (
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.transports,
        )
            .join()
            .for_each(|(trans, kin, transport)| {
                transport.objective_update(&time, trans, &map);
                transport_physics(&cow, &map, &time, trans, kin, transport);
            });
    }
}

fn transport_physics(
    coworld: &CollisionWorld,
    map: &Map,
    time: &TimeInfo,
    trans: &mut Transform,
    kin: &mut Kinematics,
    transport: &mut TransportComponent,
) {
    let direction = trans.direction();
    let speed: f32 = kin.velocity.magnitude() * kin.velocity.dot(direction).signum();
    let dot = (kin.velocity / speed).dot(direction);
    let kind = transport.kind;

    if speed > 1.0 && dot.abs() < 0.9 {
        let coeff = speed.max(1.0).min(9.0) / 9.0;
        kin.acceleration -= kin.velocity / coeff;
        return;
    }

    let pos = trans.position();

    let danger_length = (speed * speed / (2.0 * kind.deceleration())).min(40.0);

    let neighbors = coworld.query_around(pos, 12.0 + danger_length);

    let objs = neighbors.map(|obj| (obj.pos, coworld.get_obj(obj.id)));

    transport.calc_decision(map, speed, time, trans, objs);

    let speed = speed
        + ((transport.desired_speed - speed)
            .min(time.delta * kind.acceleration())
            .max(-time.delta * kind.deceleration()));

    let max_ang_vel = (speed.abs() / kind.min_turning_radius()).min(2.0);

    let delta_ang = direction.angle(transport.desired_dir);
    let mut ang = Vector2::unit_x().angle(direction);

    transport.ang_velocity += time.delta * kind.ang_acc();
    transport.ang_velocity = transport
        .ang_velocity
        .min(max_ang_vel)
        .min(3.0 * delta_ang.0.abs());

    ang.0 += delta_ang
        .0
        .min(transport.ang_velocity * time.delta)
        .max(-transport.ang_velocity * time.delta);
    let direction = vec2(ang.cos(), ang.sin());
    trans.set_direction(direction);
    kin.velocity = direction * speed;
}
