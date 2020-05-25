use crate::geometry::polyline::PolyLine;
use crate::geometry::Vec2;
use crate::map_model::{LaneID, Map, Pathfinder, Traversable};
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
    WaitUntil(f64),
    Simple(Traversable),
    Route(Route),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    /// Route is reversed, allows for efficient popping
    pub reversed_route: Vec<Traversable>,
    pub end_pos: Vec2,
    pub cur: Traversable,
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

    pub fn wait_until(x: f64) -> Self {
        Self {
            kind: ItineraryKind::WaitUntil(x),
            local_path: PolyLine::default(),
        }
    }

    pub fn route(
        cur: Traversable,
        objective: (LaneID, Vec2),
        map: &Map,
        pather: &impl Pathfinder,
    ) -> Option<Itinerary> {
        let mut reversed_route: Vec<Traversable> = pather
            .path(map, cur, objective.0)?
            .into_iter()
            .rev()
            .collect();

        reversed_route.pop(); // Remove start

        let kind = ItineraryKind::Route(Route {
            reversed_route,
            end_pos: objective.1,
            cur,
        });

        let mut it = Self {
            kind,
            local_path: PolyLine::default(),
        };
        it.advance(map);
        Some(it)
    }

    pub fn advance(&mut self, map: &Map) -> Option<Vec2> {
        let v = self.local_path.pop_first();
        if self.local_path.is_empty() {
            if let ItineraryKind::Route(r) = &mut self.kind {
                r.cur = r.reversed_route.pop()?;

                if !r.cur.is_valid(map) {
                    return v;
                }
                if r.reversed_route.is_empty() {
                    self.local_path.push(r.end_pos);
                } else {
                    self.local_path = r.cur.points(map);
                }
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

    pub fn is_terminal(&self) -> bool {
        match &self.kind {
            ItineraryKind::None | ItineraryKind::WaitUntil(_) => true,
            ItineraryKind::Simple(_) => self.remaining_points() == 1,
            ItineraryKind::Route(Route { reversed_route, .. }) => {
                reversed_route.is_empty() && self.remaining_points() == 1
            }
        }
    }

    pub fn get_point(&self) -> Option<Vec2> {
        self.local_path.first()
    }

    pub fn get_travers(&self) -> Option<&Traversable> {
        match &self.kind {
            ItineraryKind::None | ItineraryKind::WaitUntil(_) => None,
            ItineraryKind::Simple(cur) | ItineraryKind::Route(Route { cur, .. }) => Some(cur),
        }
    }

    pub fn kind(&self) -> &ItineraryKind {
        &self.kind
    }

    pub fn local_path(&self) -> &PolyLine {
        &self.local_path
    }

    pub fn has_ended(&self, time: f64) -> bool {
        match self.kind {
            ItineraryKind::WaitUntil(x) => time > x,
            _ => self.local_path.is_empty(),
        }
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
