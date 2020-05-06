use crate::geometry::polyline::PolyLine;
use crate::map_model::{IntersectionID, LaneID, Lanes, Map, TurnID};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum TraverseDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum TraverseKind {
    Lane(LaneID),
    Turn(TurnID),
}

impl TraverseKind {
    pub fn is_lane(&self) -> bool {
        matches!(self, TraverseKind::Lane(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Inspect)]
pub struct Traversable {
    pub kind: TraverseKind,
    pub dir: TraverseDirection,
}

impl Traversable {
    pub fn new(kind: TraverseKind, dir: TraverseDirection) -> Self {
        Self { kind, dir }
    }
    pub fn points(&self, m: &Map) -> PolyLine {
        let p = match self.kind {
            TraverseKind::Lane(id) => &m.lanes()[id].points,
            TraverseKind::Turn(id) => &m.intersections()[id.parent].turns[&id].points,
        };

        match self.dir {
            TraverseDirection::Forward => p.clone(),
            TraverseDirection::Backward => PolyLine::new(p.iter().copied().rev().collect()),
        }
    }

    pub fn raw_points<'a>(&self, m: &'a Map) -> &'a PolyLine {
        match self.kind {
            TraverseKind::Lane(id) => &m.lanes()[id].points,
            TraverseKind::Turn(id) => &m.intersections()[id.parent].turns[&id].points,
        }
    }

    pub fn can_pass(&self, time: u64, lanes: &Lanes) -> bool {
        match self.kind {
            TraverseKind::Lane(id) => !lanes[id].control.get_behavior(time).is_red(),
            TraverseKind::Turn(_) => true,
        }
    }

    pub fn is_valid(&self, m: &Map) -> bool {
        match self.kind {
            TraverseKind::Lane(id) => m.lanes().contains_key(id),
            TraverseKind::Turn(id) => {
                m.intersections().contains_key(id.parent)
                    && m.intersections()[id.parent].turns.contains_key(&id)
            }
        }
    }

    pub fn destination_intersection(&self, lanes: &Lanes) -> IntersectionID {
        match self.kind {
            TraverseKind::Lane(p) => match self.dir {
                TraverseDirection::Forward => lanes[p].dst,
                TraverseDirection::Backward => lanes[p].src,
            },
            TraverseKind::Turn(id) => id.parent,
        }
    }

    pub fn destination_lane(&self) -> LaneID {
        match self.kind {
            TraverseKind::Lane(p) => p,
            TraverseKind::Turn(t) => match self.dir {
                TraverseDirection::Forward => t.dst,
                TraverseDirection::Backward => t.src,
            },
        }
    }
}

enum_inspect_impl!(TraverseKind; TraverseKind::Lane(_), TraverseKind::Turn(_));
enum_inspect_impl!(TraverseDirection; TraverseDirection::Forward, TraverseDirection::Backward);
