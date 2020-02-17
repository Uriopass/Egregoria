use crate::graphs::graph::Graph;
use crate::map_model::TrafficLight;
use cgmath::MetricSpace;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavNodeID(pub usize);
impl From<usize> for NavNodeID {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NavNode {
    pub pos: Vector2<f32>,
    pub light: TrafficLight,
}

impl NavNode {
    pub fn new(pos: Vector2<f32>) -> Self {
        NavNode {
            pos,
            light: TrafficLight::Always,
        }
    }
}

pub type NavMesh = Graph<NavNodeID, NavNode>;

impl NavMesh {
    pub fn closest_node(&self, pos: Vector2<f32>) -> Option<NavNodeID> {
        let mut id: NavNodeID = *self.ids().next()?;
        let mut min_dist = self.get(id).unwrap().pos.distance2(pos);

        for (key, value) in self {
            let dist = pos.distance2(value.pos);
            if dist < min_dist {
                id = *key;
                min_dist = dist;
            }
        }
        Some(id)
    }
}
