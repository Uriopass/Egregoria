use geom::Vec2;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Component, Debug, Inspect, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Kinematics {
    pub velocity: Vec2,
    #[inspect(proxy_type = "InspectVec2", skip = true)]
    pub acceleration: Vec2,
    pub mass: f32,
}

impl Kinematics {
    pub fn from_mass(mass: f32) -> Self {
        Kinematics {
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            mass,
        }
    }
}
