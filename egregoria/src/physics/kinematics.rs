use geom::Vec2;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Inspect, Clone, Serialize, Deserialize)]
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
