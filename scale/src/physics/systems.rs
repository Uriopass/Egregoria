use crate::engine_interaction::TimeInfo;
use crate::interaction::DeletedEvent;
use crate::physics::{Collider, Kinematics, Transform};
use crate::CollisionWorld;
use scale_geom::Vec2;
use specs::prelude::{ParallelIterator, ResourceId};
use specs::shrev::EventChannel;
use specs::{
    Join, ParJoin, Read, ReadStorage, ReaderId, System, SystemData, World, WorldExt, Write,
    WriteStorage,
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
        time_it!("Kinematics update");

        let delta = data.time.delta;

        (&mut data.transforms, &mut data.kinematics)
            .par_join()
            .for_each(|(transform, kin)| {
                kin.velocity += kin.acceleration * delta;
                transform.translate(kin.velocity * delta);
                kin.acceleration = Vec2::ZERO;
            });

        for (transform, kin, collider) in
            (&mut data.transforms, &mut data.kinematics, &data.colliders).join()
        {
            data.coworld.set_position(collider.0, transform.position());
            let (_, po) = data.coworld.get_mut(collider.0).unwrap(); // Unwrap ok: handle is deleted only when entity is deleted too
            po.dir = transform.direction();
            po.speed = kin.velocity.magnitude();
        }

        for event in data.deleted.read(&mut self.reader) {
            if let Some(v) = data.colliders.get(event.e) {
                data.coworld.remove(v.0);
            }
        }

        time_it!("Coworld maintain");
        data.coworld.maintain();
    }
}
