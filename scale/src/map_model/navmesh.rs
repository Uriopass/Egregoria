use crate::graphs::Graph;
use crate::map_model::TrafficControl;
use cgmath::MetricSpace;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct NavNodeID;
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct NavNode {
    pub pos: Vector2<f32>,
    pub control: TrafficControl,
}

impl NavNode {
    pub fn new(pos: Vector2<f32>) -> Self {
        NavNode {
            pos,
            control: TrafficControl::Always,
        }
    }

    pub fn origin() -> Self {
        Self::new([0.0, 0.0].into())
    }
}

pub type NavMesh = Graph<NavNodeID, NavNode, ()>;

impl NavMesh {
    pub fn closest_node(&self, pos: Vector2<f32>) -> Option<NavNodeID> {
        let mut id: NavNodeID = self.ids().next()?;
        let mut min_dist = self.get(id).unwrap().pos.distance2(pos);

        for (key, value) in self {
            let dist = pos.distance2(value.pos);
            if dist < min_dist {
                id = key;
                min_dist = dist;
            }
        }
        Some(id)
    }
}
