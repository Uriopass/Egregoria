use crate::pedestrians::PedestrianComponent;
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
        pedestrian.objective.x = 200.0 - pedestrian.objective.x;
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

    let mut v: Vector2<f32> = dir_to_pos * pedestrian.walking_speed;

    mr.orders.get_mut(0).unwrap().as_circle_mut().color =
        if v.x > 0.0 { Color::GREEN } else { Color::BLUE };

    mr.orders[0].as_circle_mut().filled = v.x > 0.0;

    for (his_pos, his_obj) in neighs {
        if his_pos == position {
            continue;
        }

        let towards_vec = his_pos - position;
        let dist = towards_vec.magnitude();
        let towards_dir: Vector2<f32> = towards_vec / dist;

        v += -towards_dir * (-dist / 2.0).exp() * 2.0;

        if dist < 1.0 {
            mr.orders.get_mut(0).unwrap().as_circle_mut().color = Color::RED;
        }
    }

    v += 0.1 * vec2(rand::random::<f32>(), rand::random());

    let s = v.magnitude().min(1.3 * pedestrian.walking_speed);

    kin.velocity = v.normalize_to(s);
    if !kin.velocity.is_zero() {
        trans.set_direction(kin.velocity.normalize());
    }
}
