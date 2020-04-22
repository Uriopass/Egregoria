use crate::geometry::polyline::PolyLine;
use crate::geometry::Vec2;
use crate::map_model::{Map, Traversable};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Inspect, Serialize, Deserialize)]
pub struct Itinerary {
    kind: ItineraryKind,
    local_path: PolyLine,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ItineraryKind {
    None,
    Simple(Traversable),
    Route {
        cursor: usize,
        path: Vec<Traversable>,
    },
}

impl Itinerary {
    pub fn set_none(&mut self) {
        self.kind = ItineraryKind::None;
        self.local_path.clear();
    }

    pub fn set_simple(&mut self, t: Traversable, m: &Map) {
        self.kind = ItineraryKind::Simple(t);
        self.local_path = t.points(m);
    }

    pub fn set_route(&mut self, t: Vec<Traversable>, m: &Map) {
        self.kind = ItineraryKind::Route { cursor: 0, path: t };
        self.local_path.clear();
        if let Some(x) = self.get_travers() {
            self.local_path = x.points(m);
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
            ItineraryKind::Route { cursor, path } => path.get(*cursor),
        }
    }

    pub fn advance(&mut self, map: &Map) -> Option<Vec2> {
        let v = self.local_path.pop_first();
        if self.local_path.is_empty() {
            if let ItineraryKind::Route { cursor, path } = &mut self.kind {
                if *cursor < path.len() - 1 {
                    *cursor += 1;
                    self.local_path = path[*cursor].points(map);
                }
            }
        }
        v
    }

    pub fn check_validity(&mut self, map: &Map) {
        match &self.kind {
            ItineraryKind::None => {}
            ItineraryKind::Simple(id) => {
                if !id.is_valid(map) {
                    self.set_none()
                }
            }
            ItineraryKind::Route { .. } => todo!(),
        }
    }

    pub fn has_ended(&self) -> bool {
        match &self.kind {
            ItineraryKind::None => true,
            ItineraryKind::Simple(_) => self.local_path.is_empty(),
            ItineraryKind::Route { cursor, path } => *cursor >= path.len(),
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

enum_inspect_impl!(ItineraryKind; ItineraryKind::None, ItineraryKind::Simple(_), ItineraryKind::Route { .. });
