use crate::geometry::polyline::PolyLine;
use crate::map_model::Traversable;
use imgui_inspect_derive::*;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage};

#[derive(Clone, Serialize, Deserialize, Component, Inspect)]
pub struct PedestrianComponent {
    pub objective: Option<Traversable>,
    pub pos_objective: PolyLine,
    pub walking_speed: f32,
}

impl Default for PedestrianComponent {
    fn default() -> Self {
        Self {
            objective: None,
            pos_objective: PolyLine::default(),
            walking_speed: rand_distr::Normal::new(1.34f32, 0.26) // https://arxiv.org/pdf/cond-mat/9805244.pdf
                .unwrap()
                .sample(&mut rand::thread_rng())
                .max(0.5),
        }
    }
}
