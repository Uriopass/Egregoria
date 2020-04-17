use crate::engine_interaction::TimeInfo;
use crate::physics::{Collider, Kinematics, Transform};
use crate::CollisionWorld;
use cgmath::{Array, InnerSpace, Zero};
use specs::prelude::ResourceId;
use specs::{Join, Read, ReadStorage, System, SystemData, World, Write, WriteStorage};

pub struct KinematicsApply;

#[derive(SystemData)]
pub struct KinematicsApplyData<'a> {
    time: Read<'a, TimeInfo>,
    coworld: Write<'a, CollisionWorld, specs::shred::PanicHandler>,
    colliders: ReadStorage<'a, Collider>,
    transforms: WriteStorage<'a, Transform>,
    kinematics: WriteStorage<'a, Kinematics>,
}

impl<'a> System<'a> for KinematicsApply {
    type SystemData = KinematicsApplyData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let delta = data.time.delta;

        for (transform, kin, collider) in (
            &mut data.transforms,
            &mut data.kinematics,
            (&data.colliders).maybe(),
        )
            .join()
        {
            assert!(kin.velocity.is_finite());
            assert!(transform.position().is_finite());

            kin.velocity += kin.acceleration * delta;
            transform.translate(kin.velocity * delta);
            kin.acceleration.set_zero();

            if let Some(Collider(handle)) = collider {
                data.coworld.set_position(*handle, transform.position());
                let po = data.coworld.get_obj_mut(*handle);
                po.dir = transform.direction();
                po.speed = kin.velocity.magnitude();
            }
        }

        data.coworld.maintain();
    }
}
