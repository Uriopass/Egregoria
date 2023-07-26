use crate::transportation::Vehicle;
use crate::utils::par_command_buffer::ComponentDrop;
use crate::utils::resources::Resources;
use egui_inspect::Inspect;
use egui_inspect::InspectVec2Rotation;
use flat_spatial::grid::GridHandle;
use geom::Transform;
use geom::Vec2;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize, Inspect)]
pub struct Speed {
    pub speed: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsGroup {
    Unknown,
    Vehicles,
    Pedestrians,
}

debug_inspect_impl!(PhysicsGroup);

#[derive(Copy, Clone, Serialize, Deserialize, Inspect)]
pub struct PhysicsObject {
    #[inspect(proxy_type = "InspectVec2Rotation")]
    pub dir: Vec2,
    pub speed: f32,
    pub radius: f32,
    pub height: f32,
    pub group: PhysicsGroup,
    pub flag: u64,
}

impl Default for PhysicsObject {
    fn default() -> Self {
        Self {
            dir: Vec2::X,
            speed: 0.0,
            radius: 1.0,
            height: 0.0,
            group: PhysicsGroup::Unknown,
            flag: 0,
        }
    }
}

pub type CollisionWorld = flat_spatial::Grid<PhysicsObject, Vec2>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Collider(pub GridHandle);

debug_inspect_impl!(Collider);

#[profiling::function]
pub fn coworld_synchronize(world: &mut World, resources: &mut Resources) {
    let mut coworld = resources.get_mut::<CollisionWorld>().unwrap();
    world
        .query_mut::<(&Transform, &Speed, &Collider, Option<&Vehicle>)>()
        .into_iter()
        .for_each(|(_, (trans, kin, coll, v))| {
            coworld.set_position(coll.0, trans.position.xy());
            let (_, po) = coworld.get_mut(coll.0).unwrap(); // Unwrap ok: handle is deleted only when entity is deleted too
            po.dir = trans.dir.xy();
            po.speed = kin.speed;
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
