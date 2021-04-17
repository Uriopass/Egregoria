use crate::physics::{Collider, Kinematics};
use crate::utils::par_command_buffer::ComponentDrop;
use crate::utils::time::GameTime;
use crate::vehicles::Vehicle;
use crate::CollisionWorld;
use geom::Transform;
use legion::world::SubWorld;
use legion::{system, Entity, Query, Resources};

register_system!(kinematics_apply);
#[system]
pub fn kinematics_apply(
    #[resource] time: &GameTime,
    qry: &mut Query<(&mut Transform, &Kinematics)>,
    sw: &mut SubWorld,
) {
    let delta = time.delta;
    qry.par_for_each_mut(sw, |(trans, kin): (&mut Transform, &Kinematics)| {
        trans.translate(kin.velocity * delta);
    });
}

register_system!(coworld_synchronize);
#[system]
pub fn coworld_synchronize(
    #[resource] coworld: &mut CollisionWorld,
    qry: &mut Query<(&Transform, &Kinematics, &Collider, Option<&Vehicle>)>,
    sw: &SubWorld,
) {
    qry.for_each(sw, |(trans, kin, coll, v)| {
        coworld.set_position(coll.0, trans.position());
        let (_, po) = coworld.get_mut(coll.0).unwrap(); // Unwrap ok: handle is deleted only when entity is deleted too
        po.dir = trans.direction();
        po.speed = kin.velocity.magnitude();
        if let Some(v) = v {
            po.flag = v.flag;
        }
    });
    coworld.maintain();
}

impl ComponentDrop for Collider {
    fn drop(&mut self, res: &mut Resources, _: Entity) {
        res.get_mut::<CollisionWorld>()
            .unwrap()
            .remove_maintain(self.0);
    }
}
