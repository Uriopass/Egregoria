use geom::Vec2;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Inspect)]
pub struct Kinematics {
    pub velocity: Vec2,
    pub mass: f32,
}

impl Kinematics {
    pub fn from_mass(mass: f32) -> Self {
        Kinematics {
            velocity: Vec2::ZERO,
            mass,
        }
    }
}
