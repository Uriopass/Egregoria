use crate::graphs::graph::NodeID;
use cgmath::Vector2;

use specs::Entity;
use std::collections::HashMap;

mod road_graph;
mod road_graph_synchronize;

pub use road_graph::RoadGraph;
pub use road_graph_synchronize::RoadGraphSynchronize;

#[derive(Clone, Copy)]
pub struct RoadNode {
    pub pos: Vector2<f32>,
    pub e: Option<Entity>,
}

impl RoadNode {
    pub fn new(pos: Vector2<f32>) -> Self {
        RoadNode { pos, e: None }
    }
}

pub struct Intersection {
    pub pos: Vector2<f32>,
    pub out_nodes: HashMap<NodeID, NodeID>,
    pub in_nodes: HashMap<NodeID, NodeID>,
    pub e: Option<Entity>,
}

impl Intersection {
    pub fn new(pos: Vector2<f32>) -> Self {
        Intersection {
            pos,
            out_nodes: HashMap::new(),
            in_nodes: HashMap::new(),
            e: None,
        }
    }
}
