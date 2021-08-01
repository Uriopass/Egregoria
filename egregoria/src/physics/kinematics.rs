use geom::Vec3;
use imgui_inspect_derive::Inspect;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize, Inspect)]
pub struct Kinematics {
    pub velocity: Vec3,
}
