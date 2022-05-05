use crate::physics::{Collider, Kinematics};
use crate::utils::par_command_buffer::ComponentDrop;
use crate::utils::time::GameTime;
use crate::vehicles::Vehicle;
use crate::CollisionWorld;
use geom::Transform;
use hecs::{Entity, World};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use resources::Resources;

pub fn kinematics_apply(world: &mut World, resources: &mut Resources) {
    let time = resources.get::<GameTime>().unwrap();
    let delta = time.delta;
    world
        .query_mut::<(&mut Transform, &Kinematics)>()
        .into_iter()
        .par_bridge()
        .for_each(|(_, (trans, kin))| {
            trans.position += kin.velocity * delta;
        });
}

pub fn coworld_synchronize(world: &mut World, resources: &mut Resources) {
    let mut coworld = resources.get_mut::<CollisionWorld>().unwrap();
    world
        .query_mut::<(&Transform, &Kinematics, &Collider, Option<&Vehicle>)>()
        .into_iter()
        .for_each(|(_, (trans, kin, coll, v))| {
            coworld.set_position(coll.0, trans.position.xy());
            let (_, po) = coworld.get_mut(coll.0).unwrap(); // Unwrap ok: handle is deleted only when entity is deleted too
            po.dir = trans.dir.xy();
            po.speed = kin.velocity.magnitude();
            po.height = trans.position.z;
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
