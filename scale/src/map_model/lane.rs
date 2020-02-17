use crate::map_model::{NavNodeID, Road, RoadID};
use serde::{Deserialize, Serialize};
use slab::Slab;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneID(pub usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LaneType {
    Driving,
    Biking,
    Bus,
    Construction,
}

#[derive(Serialize, Deserialize)]
pub struct Lane {
    pub id: LaneID,
    pub parent: RoadID,
    pub lane_type: LaneType,
    pub src_node: Option<NavNodeID>,
    pub dst_node: Option<NavNodeID>,
}

impl Lane {
    pub fn make_forward(store: &mut Slab<Lane>, owner: &mut Road, lane_type: LaneType) {
        let entry = store.vacant_entry();
        let id = LaneID(entry.key());
        entry.insert(Lane {
            id,
            parent: owner.id(),
            lane_type,
            src_node: None,
            dst_node: None,
        });
        owner.lanes_forward.push(id)
    }

    pub fn make_backward(store: &mut Slab<Lane>, owner: &mut Road, lane_type: LaneType) {
        let entry = store.vacant_entry();
        let id = LaneID(entry.key());
        entry.insert(Lane {
            id,
            parent: owner.id(),
            lane_type,
            src_node: None,
            dst_node: None,
        });
        owner.lanes_backward.push(id)
    }
}
