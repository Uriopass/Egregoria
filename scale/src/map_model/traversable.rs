use crate::map_model::{LaneID, Lanes, Map, TurnID};
use cgmath::Vector2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Traversable {
    Lane(LaneID),
    Turn(TurnID),
}

impl Traversable {
    pub fn points<'a>(&self, m: &'a Map) -> &'a [Vector2<f32>] {
        match *self {
            Traversable::Lane(id) => m.lanes()[id].points.as_slice(),
            Traversable::Turn(id) => m.intersections()[id.parent].turns[&id].points.as_slice(),
        }
    }

    pub fn can_pass(&self, time: u64, lanes: &Lanes) -> bool {
        match self {
            Traversable::Lane(id) => !lanes[*id].control.get_behavior(time).is_red(),
            Traversable::Turn(_) => true,
        }
    }

    pub fn is_valid(&self, m: &Map) -> bool {
        match *self {
            Traversable::Lane(id) => m.lanes().contains_key(id),
            Traversable::Turn(id) => {
                m.intersections().contains_key(id.parent)
                    && m.intersections()[id.parent].turns.contains_key(&id)
            }
        }
    }
}
