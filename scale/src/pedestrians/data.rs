use crate::gui::InspectVec2;
use cgmath::{vec2, Vector2};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage};

pub(crate) const WALKING_SPEED: f32 = 3.0;

#[derive(Clone, Serialize, Deserialize, Component, Inspect)]
pub struct PedestrianComponent {
    #[inspect(proxy_type = "InspectVec2")]
    pub objective: Vector2<f32>,
}

impl Default for PedestrianComponent {
    fn default() -> Self {
        Self {
            objective: vec2(0.0, 0.0),
        }
    }
}
