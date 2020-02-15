use crate::engine_interaction::TimeInfo;
use crate::physics::{Collider, Kinematics, Transform};
use crate::PhysicsWorld;
use cgmath::Zero;
use nalgebra as na;
use nalgebra::Isometry2;
use specs::prelude::ResourceId;
use specs::{Join, Read, ReadStorage, System, SystemData, World, Write, WriteStorage};

pub struct KinematicsApply;

#[derive(SystemData)]
pub struct KinematicsApplyData<'a> {
    time: Read<'a, TimeInfo>,
    colliders: ReadStorage<'a, Collider>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
    coworld: Write<'a, PhysicsWorld, specs::shred::PanicHandler>,
}

impl<'a> System<'a> for KinematicsApply {
    type SystemData = KinematicsApplyData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let delta = data.time.delta;

        for (transform, kin) in (&mut data.transforms, &mut data.kinematics).join() {
            kin.velocity += kin.acceleration * delta;
            transform.translate(kin.velocity * delta);
            kin.acceleration.set_zero();
        }

        for (transform, collider) in (&data.transforms, &data.colliders).join() {
            let collision_obj = data
                .coworld
                .get_mut(collider.0)
                .expect("Invalid collision object; was it removed from ncollide but not specs?");
            let p = transform.position();
            let iso = Isometry2::from_parts(
                na::Translation2::new(p.x, p.y),
                na::UnitComplex::new_unchecked(na::Complex::new(
                    transform.get_cos(),
                    transform.get_sin(),
                )),
            );

            collision_obj.set_position(iso);
        }
    }
}
