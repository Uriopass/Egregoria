use crate::map_model::traffic_lights::TrafficLight;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use specs::World;

mod intersection;
mod map;
mod road_graph;
mod road_graph_synchronize;
mod saveload;
mod traffic_lights;

pub use intersection::*;
pub use map::*;
pub use road_graph::RoadGraph;
pub use road_graph_synchronize::*;
pub use saveload::*;
pub use traffic_lights::*;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadNodeID(pub usize);
impl From<usize> for RoadNodeID {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RoadNode {
    pub pos: Vector2<f32>,
    pub light: TrafficLight,
}

impl RoadNode {
    pub fn new(pos: Vector2<f32>) -> Self {
        RoadNode {
            pos,
            light: TrafficLight::Always,
        }
    }
}

pub fn setup(world: &mut World) {
    load(world);
}
