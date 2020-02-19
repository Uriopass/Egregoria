use crate::map_model::{IntersectionID, NavMesh, NavNodeID, RoadID};
use cgmath::InnerSpace;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LaneID;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LaneType {
    Driving,
    Biking,
    Bus,
    Construction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaneDirection {
    Forward,
    Backward,
}

#[derive(Serialize, Deserialize)]
pub struct Lane {
    pub id: LaneID,
    pub parent: RoadID,
    pub lane_type: LaneType,

    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,

    pub src_node: Option<NavNodeID>,
    pub dst_node: Option<NavNodeID>,

    pub direction: LaneDirection,
}

impl Lane {
    pub fn set_inter_node(&mut self, id: IntersectionID, node: NavNodeID) {
        if id == self.src_i {
            self.src_node = Some(node)
        } else if id == self.dst_i {
            self.dst_node = Some(node)
        } else {
            panic!("Trying to assign node to not corresponding intersection");
        }
    }

    pub fn get_inter_node(&self, id: IntersectionID) -> NavNodeID {
        if id == self.src_i {
            self.src_node
        } else if id == self.dst_i {
            self.dst_node
        } else {
            panic!("Trying to get node to not corresponding intersection");
        }
        .expect("Lane not generated yet")
    }

    pub fn get_orientation_vec(&self, mesh: &NavMesh) -> Vector2<f32> {
        let src = mesh[self.src_node.unwrap()].pos;
        let dst = mesh[self.dst_node.unwrap()].pos;

        assert_ne!(dst, src);

        let vec = (dst - src).normalize();
        match self.direction {
            LaneDirection::Forward => vec,
            LaneDirection::Backward => -vec,
        }
    }
}
