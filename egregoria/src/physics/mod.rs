use egui_inspect::InspectVec2Rotation;
use egui_inspect_derive::Inspect;
use flat_spatial::grid::GridHandle;
use serde::{Deserialize, Serialize};

mod kinematics;
pub mod systems;

use geom::Vec2;
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
