use crate::geometry::Vec2;
use crate::gui::InspectVec2;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Component, Debug, Inspect, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Kinematics {
    #[inspect(proxy_type = "InspectVec2")]
    pub velocity: Vec2,
    #[inspect(proxy_type = "InspectVec2", skip = true)]
    pub acceleration: Vec2,
    pub mass: f32,
}

impl Kinematics {
    pub fn from_mass(mass: f32) -> Self {
        Kinematics {
            velocity: Vec2::zero(),
            acceleration: Vec2::zero(),
            mass,
        }
    }
}
