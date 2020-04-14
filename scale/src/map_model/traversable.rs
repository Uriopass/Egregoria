use crate::geometry::polyline::PolyLine;
use crate::map_model::{IntersectionID, LaneID, Lanes, Map, TurnID};
use imgui_inspect::InspectRenderDefault;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Traversable {
    Lane(LaneID),
    Turn(TurnID),
}

impl Traversable {
    pub fn points_from<'a>(&self, m: &'a Map, i: IntersectionID) -> PolyLine {
        match *self {
            Traversable::Lane(id) => {
                let l = &m.lanes()[id];
                if l.src == i {
                    l.points.clone()
                } else {
                    PolyLine::new(l.points.iter().rev().copied().collect())
                }
            }
            Traversable::Turn(id) => m.intersections()[id.parent].turns[&id].points.clone(),
        }
    }

    pub fn points<'a>(&self, m: &'a Map) -> PolyLine {
        match *self {
            Traversable::Lane(id) => m.lanes()[id].points.clone(),
            Traversable::Turn(id) => m.intersections()[id.parent].turns[&id].points.clone(),
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

enum_inspect_impl!(Traversable; Traversable::Lane(_), Traversable::Turn(_));
