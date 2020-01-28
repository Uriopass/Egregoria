use crate::graphs::graph::NodeID;
use crate::map::traffic_lights::TrafficLight;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use specs::storage::BTreeStorage;
use specs::{Component, LazyUpdate, World, WorldExt};
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

const GRAPH_FILENAME: &str = "world/graph";

pub fn save(world: &mut World) {
    world.read_resource::<RoadGraph>().save(GRAPH_FILENAME);
}

pub fn load(world: &mut World) {
    let rg = RoadGraph::from_file(GRAPH_FILENAME).unwrap_or_else(RoadGraph::empty);
    for (inter_id, inter) in rg.intersections() {
        make_inter_entity(
            *inter_id,
            inter.pos,
            &world.read_resource::<LazyUpdate>(),
            &world.entities(),
        );
    }
    world.insert(rg);
}

pub fn setup(world: &mut World) {
    load(world);
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

#[derive(Component, Clone, Serialize, Deserialize)]
#[storage(BTreeStorage)]
pub struct IntersectionComponent {
    pub id: NodeID,
}
empty_inspect_impl!(IntersectionComponent);
