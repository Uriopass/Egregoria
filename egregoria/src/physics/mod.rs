use crate::Egregoria;
use flat_spatial::grid::GridHandle;
use geom::Vec2;
use imgui::Ui;
use imgui_inspect::InspectDragf;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault, InspectVec2Rotation};
use imgui_inspect_derive::*;
use legion::IntoQuery;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod kinematics;
pub mod systems;

pub use kinematics::*;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhysicsGroup {
    Unknown,
    Vehicles,
    Pedestrians,
}

enum_inspect_impl!(PhysicsGroup; PhysicsGroup::Unknown, PhysicsGroup::Vehicles, PhysicsGroup::Pedestrians);

#[derive(Clone, Copy, Inspect, Serialize, Deserialize)]
pub struct PhysicsObject {
    #[inspect(proxy_type = "InspectVec2Rotation")]
    pub dir: Vec2,
    pub speed: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub radius: f32,
    pub group: PhysicsGroup,
}

impl Default for PhysicsObject {
    fn default() -> Self {
        Self {
            dir: Vec2::UNIT_X,
            speed: 0.0,
            radius: 1.0,
            group: PhysicsGroup::Unknown,
        }
    }
}

pub type CollisionWorld = flat_spatial::SparseGrid<PhysicsObject>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Collider(pub GridHandle);

impl InspectRenderDefault<Collider> for Collider {
    fn render(_: &[&Collider], _: &'static str, _: &Ui, _: &InspectArgsDefault) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut Collider],
        label: &'static str,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!()
        }
        let d = &mut data[0];
        ui.text(format!("{:?} {}", d.0, label));
        false
    }
}

type SerPhysicsObj<'a> = (GridHandle, ([f32; 2], PhysicsObject));

pub fn serialize_colliders(state: &mut Egregoria) {
    let coworld = &*state.read::<CollisionWorld>();

    let mut objs: Vec<SerPhysicsObj> = vec![];
    for &h in <&Collider>::query().iter(&state.world) {
        let (pos, pobj) = unwrap_or!(coworld.get(h.0), return);
        objs.push((h.0, ([pos.x, pos.y], *pobj)));
    }
    common::saveload::save(&objs, "coworld");
}

pub fn deserialize_colliders(state: &mut Egregoria) -> Option<()> {
    let objs: Vec<SerPhysicsObj> = common::saveload::load("coworld")?;

    let coworld: &mut CollisionWorld = &mut *state.resources.get_mut::<CollisionWorld>().unwrap();

    let mut handle_map: HashMap<GridHandle, GridHandle> = HashMap::default();

    for (e, (p, obj)) in objs {
        let h = coworld.insert(p, obj);
        handle_map.insert(e, h);
    }

    for c in <&mut Collider>::query().iter_mut(&mut state.world) {
        c.0 = handle_map[&c.0];
    }

    Some(())
}
