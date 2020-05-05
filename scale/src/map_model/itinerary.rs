use crate::geometry::polyline::PolyLine;
use crate::geometry::Vec2;
use crate::map_model::{IntersectionID, LaneID, Map, Traversable};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Inspect, Serialize, Deserialize)]
pub struct Itinerary {
    kind: ItineraryKind,
    local_path: PolyLine,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItineraryKind {
    None,
    Simple(Traversable),
    Route(Route),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    /// End is at the beginning, allows for efficient popping
    reversed_route: Vec<IntersectionID>,
    end: LaneID,
    end_pos: Vec2,
    cur: Traversable,
}

impl Itinerary {
    pub fn none() -> Self {
        Self {
            kind: ItineraryKind::None,
            local_path: PolyLine::default(),
        }
    }

    pub fn simple(t: Traversable, m: &Map) -> Self {
        Self {
            kind: ItineraryKind::Simple(t),
            local_path: t.points(m),
        }
    }

    pub fn route(
        path: Vec<IntersectionID>,
        cur: Traversable,
        objective: (LaneID, Vec2),
        m: &Map,
    ) -> Itinerary {
        let kind = ItineraryKind::Route(Route {
            reversed_route: path.into_iter().rev().collect(),
            end: objective.0,
            end_pos: objective.1,
            cur,
        });

        Self {
            kind,
            local_path: cur.points(m),
        }
    }

    pub fn advance(&mut self, map: &Map) -> Option<Vec2> {
        let v = self.local_path.pop_first();
        if self.local_path.is_empty() {
            if let ItineraryKind::Route(r) = &mut self.kind {
                // ...
            }
        }
        v
    }

    pub fn check_validity(&mut self, map: &Map) {
        if let Some(false) = self.get_travers().map(|x| x.is_valid(map)) {
            self.kind = ItineraryKind::None;
            self.local_path.clear();
        }
    }

    pub fn remaining_points(&self) -> usize {
        self.local_path.n_points()
    }

    pub fn get_point(&self) -> Option<Vec2> {
        self.local_path.first()
    }

    pub fn get_travers(&self) -> Option<&Traversable> {
        match &self.kind {
            ItineraryKind::None => None,
            ItineraryKind::Simple(x) => Some(x),
            ItineraryKind::Route(Route { cur, .. }) => Some(cur),
        }
    }

    pub fn has_ended(&self) -> bool {
        self.local_path.is_empty()
    }

    pub fn is_none(&self) -> bool {
        matches!(self.kind, ItineraryKind::None)
    }
}

impl Default for ItineraryKind {
    fn default() -> Self {
        ItineraryKind::None
    }
}

enum_inspect_impl!(ItineraryKind; ItineraryKind::None, ItineraryKind::Simple(_));
