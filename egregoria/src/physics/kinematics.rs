use geom::Vec2;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize, Inspect)]
pub struct Kinematics {
    pub velocity: Vec2,
}
