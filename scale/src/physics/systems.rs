use crate::engine_interaction::TimeInfo;
use crate::geometry::Vec2;
use crate::interaction::DeletedEvent;
use crate::physics::{Collider, Kinematics, Transform};
use crate::CollisionWorld;
use specs::prelude::ResourceId;
use specs::shrev::EventChannel;
use specs::{
    Join, Read, ReadStorage, ReaderId, System, SystemData, World, WorldExt, Write, WriteStorage,
};

pub struct KinematicsApply {
    reader: ReaderId<DeletedEvent>,
}

impl KinematicsApply {
    pub fn new(world: &mut World) -> KinematicsApply {
        let reader = world
            .write_resource::<EventChannel<DeletedEvent>>()
            .register_reader();

        Self { reader }
    }
}

#[derive(SystemData)]
pub struct KinematicsApplyData<'a> {
    time: Read<'a, TimeInfo>,
    deleted: Read<'a, EventChannel<DeletedEvent>>,
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
            kin.velocity += kin.acceleration * delta;
            transform.translate(kin.velocity * delta);
            kin.acceleration = Vec2::zero();

            if let Some(Collider(handle)) = collider {
                data.coworld.set_position(*handle, transform.position());
                let (_, po) = data.coworld.get_mut(*handle).unwrap();
                po.dir = transform.direction();
                po.speed = kin.velocity.magnitude();
            }
        }

        for event in data.deleted.read(&mut self.reader) {
            if let Some(v) = data.colliders.get(event.e) {
                data.coworld.remove(v.0);
            }
        }

        data.coworld.maintain();
    }
}
