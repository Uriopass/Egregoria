use crate::gui::{InspectDragf, InspectVec2Rotation};
use flat_spatial::grid::GridHandle;
use geom::Vec2;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::*;
use specs::{Component, VecStorage};

pub mod systems;

mod kinematics;
mod transform;

pub use kinematics::*;
pub use transform::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PhysicsGroup {
    Unknown,
    Vehicles,
    Pedestrians,
}

enum_inspect_impl!(PhysicsGroup; PhysicsGroup::Unknown, PhysicsGroup::Vehicles, PhysicsGroup::Pedestrians);

#[derive(Clone, Copy, Inspect)]
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

#[derive(Clone, Component, Debug)]
#[storage(VecStorage)]
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
