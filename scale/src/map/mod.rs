use crate::graphs::graph::NodeID;
use crate::map::traffic_lights::TrafficLight;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod road_graph;
mod road_graph_synchronize;
mod traffic_lights;

pub use road_graph::RoadGraph;
pub use road_graph_synchronize::*;
pub use traffic_lights::*;

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

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub pos: Vector2<f32>,
    pub out_nodes: HashMap<NodeID, NodeID>,
    pub in_nodes: HashMap<NodeID, NodeID>,
}

impl Intersection {
    pub fn new(pos: Vector2<f32>) -> Self {
        Intersection {
            pos,
            out_nodes: HashMap::new(),
            in_nodes: HashMap::new(),
        }
    }
}
