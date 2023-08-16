use crate::transportation::Vehicle;
use crate::utils::resources::Resources;
use crate::{Egregoria, World};
use egui_inspect::Inspect;
use egui_inspect::InspectVec2Rotation;
use flat_spatial::grid::GridHandle;
use geom::Transform;
use geom::Vec2;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Default, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Speed(pub f32);

impl Debug for Speed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}m/s", self.0)
    }
}

debug_inspect_impl!(Speed);

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

impl Collider {
    pub fn destroy(self) -> impl FnOnce(&mut Egregoria) {
        move |goria| {
            let cw = &mut goria.write::<CollisionWorld>();
            cw.remove_maintain(self.0);
        }
    }
}

pub fn coworld_synchronize(world: &mut World, resources: &mut Resources) {
    profiling::scope!("physics::coworld_synchronize");
    let mut coworld = resources.write::<CollisionWorld>();

    world.query_trans_speed_coll_vehicle().for_each(
        |(trans, kin, coll, v): (&Transform, &Speed, Collider, Option<&Vehicle>)| {
            coworld.set_position(coll.0, trans.position.xy());
            let (_, po) = coworld.get_mut(coll.0).unwrap(); // Unwrap ok: handle is deleted only when entity is deleted too
            po.dir = trans.dir.xy();
            po.speed = kin.0;
            po.height = trans.position.z;
            if let Some(v) = v {
                po.flag = v.flag;
            }
        },
    );

    coworld.maintain();
}
