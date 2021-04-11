use flat_spatial::grid::GridHandle;
use geom::Vec2;
use imgui::Ui;
use imgui_inspect::InspectDragf;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault, InspectVec2Rotation};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

mod kinematics;
pub mod systems;

pub use kinematics::*;

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
    #[inspect(proxy_type = "InspectDragf")]
    pub radius: f32,
    pub group: PhysicsGroup,
    pub flag: u64,
}

impl Default for PhysicsObject {
    fn default() -> Self {
        Self {
            dir: Vec2::UNIT_X,
            speed: 0.0,
            radius: 1.0,
            group: PhysicsGroup::Unknown,
            flag: 0,
        }
    }
}

pub type CollisionWorld = flat_spatial::SparseGrid<PhysicsObject>;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Collider(pub GridHandle);

impl InspectRenderDefault<Collider> for Collider {
    fn render(data: &[&Collider], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        let d = unwrap_ret!(data.get(0));
        ui.text(format!("{:?} {}", d.0, label));
    }

    fn render_mut(
        data: &mut [&mut Collider],
        label: &'static str,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            panic!()
        }
        let d = unwrap_ret!(data.get_mut(0), false);
        ui.text(format!("{:?} {}", d.0, label));
        false
    }
}
