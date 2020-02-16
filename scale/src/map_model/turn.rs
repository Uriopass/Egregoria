use crate::map_model::{IntersectionID, LaneID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnID(pub usize);

#[derive(Serialize, Deserialize)]
pub struct Turn {
    pub parent: IntersectionID,
    pub src: LaneID,
    pub dst: LaneID,
}
