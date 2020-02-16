use crate::map_model::IntersectionID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadID(pub usize);

#[derive(Serialize, Deserialize)]
pub struct Road {
    pub id: RoadID,
    pub a: IntersectionID,
    pub b: IntersectionID,
}
