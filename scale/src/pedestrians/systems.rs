use crate::pedestrians::{PedestrianComponent, WALKING_SPEED};
use crate::physics::{CollisionWorld, Kinematics, PhysicsObject, Transform};
use crate::rendering::meshrender_component::MeshRender;
use crate::rendering::Color;
use cgmath::{vec2, InnerSpace, MetricSpace, Vector2, Zero};
use specs::prelude::*;
use specs::shred::PanicHandler;
use std::borrow::Borrow;

#[derive(Default)]
pub struct PedestrianDecision;

#[derive(SystemData)]
pub struct PedestrianDecisionData<'a> {
    cow: Read<'a, CollisionWorld, PanicHandler>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    pedestrians: WriteStorage<'a, PedestrianComponent>,
    meshrender: WriteStorage<'a, MeshRender>,
}

impl<'a> System<'a> for PedestrianDecision {
    type SystemData = PedestrianDecisionData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let cow: &CollisionWorld = data.cow.borrow();

        (
            &mut data.transforms,
            &mut data.kinematics,
            &mut data.pedestrians,
            &mut data.meshrender,
        )
            .join()
            .for_each(|(trans, kin, pedestrian, mr)| {
                objective_update(pedestrian, trans);

                let neighbors = cow.query_around(trans.position(), 10.0);

                let objs = neighbors.map(|obj| (obj.pos, cow.get_obj(obj.id)));

                calc_decision(pedestrian, trans, kin, objs, mr);
            });
    }
}

pub fn objective_update(pedestrian: &mut PedestrianComponent, trans: &Transform) {
    if pedestrian.objective.distance(trans.position()) < 2.0 {
        pedestrian.objective = 200.0f32 * vec2(rand::random(), rand::random());
    }
}

pub fn calc_decision<'a>(
    pedestrian: &mut PedestrianComponent,
    trans: &mut Transform,
    kin: &mut Kinematics,
    neighs: impl Iterator<Item = (Vector2<f32>, &'a PhysicsObject)>,
    mr: &mut MeshRender,
) {
    let objective = pedestrian.objective;
    let position = trans.position();

    let delta_pos: Vector2<f32> = objective - position;
    let dist_to_pos = delta_pos.magnitude();
    let dir_to_pos: Vector2<f32> = delta_pos / dist_to_pos;

    let v: Vector2<f32> = dir_to_pos * WALKING_SPEED;

    mr.orders.get_mut(0).unwrap().as_circle_mut().color = Color::WHITE;
    for (his_pos, his_obj) in neighs {
        if his_pos.distance2(position) < 1.0 {
            mr.orders.get_mut(0).unwrap().as_circle_mut().color = Color::RED;
        }
    }

    kin.velocity = v;
    if !kin.velocity.is_zero() {
        trans.set_direction(kin.velocity.normalize());
    }
}
