use crate::RoadID;
use geom::obb::OBB;
use geom::segment::Segment;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LotID;
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Lot {
    pub id: LotID,
    pub parent: RoadID,
    pub shape: OBB,
    pub road_edge: Segment,
}
