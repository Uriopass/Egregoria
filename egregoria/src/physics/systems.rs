use crate::physics::{Collider, Kinematics};
use crate::utils::par_command_buffer::ComponentDrop;
use crate::utils::time::GameTime;
use crate::vehicles::Vehicle;
use crate::CollisionWorld;
use geom::Transform;
use legion::{system, Entity, Resources};

register_system!(kinematics_apply);
#[system(par_for_each)]
pub fn kinematics_apply(
    #[resource] time: &GameTime,
    transform: &mut Transform,
    kin: &mut Kinematics,
) {
    transform.translate(kin.velocity * time.delta);
}

register_system!(coworld_synchronize);
#[system(for_each)]
pub fn coworld_synchronize(
    #[resource] coworld: &mut CollisionWorld,
    transform: &Transform,
    kin: &Kinematics,
    collider: &Collider,
    v: Option<&Vehicle>,
) {
    coworld.set_position(collider.0, transform.position());
    let (_, po) = coworld.get_mut(collider.0).unwrap(); // Unwrap ok: handle is deleted only when entity is deleted too
    po.dir = transform.direction();
    po.speed = kin.velocity.magnitude();
    if let Some(v) = v {
        po.flag = v.flag;
    }
    coworld.maintain();
}

impl ComponentDrop for Collider {
    fn drop(&mut self, res: &mut Resources, _: Entity) {
        res.get_mut::<CollisionWorld>()
            .unwrap()
            .remove_maintain(self.0);
    }
}
