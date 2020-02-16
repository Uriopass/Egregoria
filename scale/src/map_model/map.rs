use crate::map_model::{Intersection, IntersectionID, Lane, LaneID, Road, RoadID, Turn, TurnID};
use std::collections::HashMap;

pub struct Map {
    pub roads: HashMap<RoadID, Road>,
    pub lanes: HashMap<LaneID, Lane>,
    pub intersections: HashMap<IntersectionID, Intersection>,
    pub turns: HashMap<TurnID, Turn>,
}
