use crate::engine_interaction::TimeInfo;
use crate::physics::{Collider, Kinematics};
use crate::{CollisionWorld, Deleted};
use geom::{Transform, Vec2};
use legion::system;

#[system(for_each)]
pub fn kinematics_apply(
    #[resource] time: &TimeInfo,
    transform: &mut Transform,
    kin: &mut Kinematics,
) {
    kin.velocity += kin.acceleration * time.delta;
    transform.translate(kin.velocity * time.delta);
    kin.acceleration = Vec2::ZERO;
}

#[system(for_each)]
pub fn coworld_synchronize(
    #[resource] coworld: &mut CollisionWorld,
    transform: &Transform,
    kin: &Kinematics,
    collider: &Collider,
) {
    coworld.set_position(collider.0, transform.position());
    let (_, po) = coworld.get_mut(collider.0).unwrap(); // Unwrap ok: handle is deleted only when entity is deleted too
    po.dir = transform.direction();
    po.speed = kin.velocity.magnitude();
}

#[system]
pub fn coworld_maintain(
    #[resource] coworld: &mut CollisionWorld,
    #[resource] evts: &mut Deleted<Collider>,
) {
    for Collider(handle) in evts.drain() {
        coworld.remove(handle);
    }

    coworld.maintain();
}
