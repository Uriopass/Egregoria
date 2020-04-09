use crate::gui::InspectVec2;
use cgmath::{vec2, Vector2};
use imgui_inspect_derive::*;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage};

#[derive(Clone, Serialize, Deserialize, Component, Inspect)]
pub struct PedestrianComponent {
    #[inspect(proxy_type = "InspectVec2")]
    pub objective: Vector2<f32>,
    pub walking_speed: f32,
}

impl Default for PedestrianComponent {
    fn default() -> Self {
        Self {
            objective: vec2(0.0, 0.0),
            walking_speed: rand_distr::Normal::new(1.34f32, 0.26) // https://arxiv.org/pdf/cond-mat/9805244.pdf
                .unwrap()
                .sample(&mut rand::thread_rng())
                .max(0.5),
        }
    }
}
