use crate::engine_interaction::TimeInfo;
use crate::geometry::Vec2;
use crate::gui::InspectVec;
use crate::map_model::{LaneID, Map, Pathfinder, Traversable, TraverseKind};
use crate::physics::Transform;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::Component;

#[derive(Component, Debug, Default, Inspect, Serialize, Deserialize)]
pub struct Itinerary {
    kind: ItineraryKind,
    #[inspect(proxy_type = "InspectVec<Vec2>")]
    local_path: Vec<Vec2>,
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

pub const OBJECTIVE_OK_DIST: f32 = 4.0;

impl Itinerary {
    pub fn none() -> Self {
        Self {
            kind: ItineraryKind::None,
            local_path: Default::default(),
        }
    }

    pub fn simple(t: Traversable, m: &Map) -> Self {
        Self {
            kind: ItineraryKind::Simple(t),
            local_path: t.points(m).into_vec(),
        }
    }

    pub fn wait_until(x: f64) -> Self {
        Self {
            kind: ItineraryKind::WaitUntil(x),
            local_path: Default::default(),
        }
    }

    pub fn route(
        pos: Vec2,
        cur: Traversable,
        (l_obj, obj): (LaneID, Vec2),
        map: &Map,
        pather: &impl Pathfinder,
    ) -> Option<Itinerary> {
        let points = cur.points(map);
        let (_, segid) = points.project_segment(pos);

        if let TraverseKind::Lane(id) = cur.kind {
            if id == l_obj {
                // start lane is objective lane

                let (_, segid_obj) = points.project_segment(obj);

                if segid_obj > segid
                    || (segid_obj == segid
                        && points[segid - 1].distance2(pos) < points[segid - 1].distance2(obj))
                {
                    let kind = ItineraryKind::Route(Route {
                        reversed_route: vec![],
                        end_pos: obj,
                        cur,
                    });

                    let mut local_path = vec![];
                    local_path.extend(&points.as_slice()[segid..segid_obj]);
                    local_path.push(obj);

                    return Some(Itinerary { kind, local_path });
                }
            }
        }

        let mut points = points.into_vec();
        points.drain(..segid - 1);

        let mut reversed_route: Vec<Traversable> =
            pather.path(map, cur, l_obj)?.into_iter().rev().collect();

        reversed_route.pop(); // Remove start

        let kind = ItineraryKind::Route(Route {
            reversed_route,
            end_pos: obj,
            cur,
        });

        let mut it = Self {
            kind,
            local_path: points,
        };
        it.advance(map);
        Some(it)
    }

    pub fn advance(&mut self, map: &Map) -> Option<Vec2> {
        let v = if self.local_path.is_empty() {
            None
        } else {
            Some(self.local_path.remove(0))
        };

        if self.local_path.is_empty() {
            if let ItineraryKind::Route(r) = &mut self.kind {
                r.cur = r.reversed_route.pop()?;

                if !r.cur.is_valid(map) {
                    return v;
                }

                let points = r.cur.points(map);
                if r.reversed_route.is_empty() {
                    let (_, id) = points.project_segment(r.end_pos);
                    self.local_path.extend(&points.as_slice()[..id]);
                    self.local_path.push(r.end_pos);
                } else {
                    self.local_path = points.into_vec();
                }
            }
        }
        v
    }

    pub fn update(&mut self, position: Vec2, time: &TimeInfo, map: &Map) {
        self.check_validity(map);

        if let Some(p) = self.get_point() {
            let dist = p.distance2(position);
            if self.is_terminal() {
                if dist < 1.0 {
                    self.advance(map);
                }
                return;
            }

            if dist < OBJECTIVE_OK_DIST * OBJECTIVE_OK_DIST {
                let k = self.get_travers().unwrap(); // Unwrap ok: We just called check_validity and get_point
                if self.remaining_points() > 1 || k.can_pass(time.time_seconds, map.lanes()) {
                    self.advance(map);
                }
            }
        }
    }

    pub fn check_validity(&mut self, map: &Map) {
        if let Some(false) = self.get_travers().map(|x| x.is_valid(map)) {
            self.kind = ItineraryKind::None;
            self.local_path.clear();
        }
    }

    pub fn remaining_points(&self) -> usize {
        self.local_path.len()
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
        self.local_path.first().copied()
    }

    pub fn get_terminal(&self) -> Option<Vec2> {
        match &self.kind {
            ItineraryKind::None | ItineraryKind::WaitUntil(_) => None,
            ItineraryKind::Simple(_) => self.local_path.last().copied(),
            ItineraryKind::Route(Route { end_pos, .. }) => Some(*end_pos),
        }
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

    pub fn local_path(&self) -> &[Vec2] {
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

enum_inspect_impl!(ItineraryKind; ItineraryKind::None, ItineraryKind::Simple(_), ItineraryKind::WaitUntil(_), ItineraryKind::Route(_));

pub struct ItinerarySystem;

#[derive(SystemData)]
pub struct ItinerarySystemData<'a> {
    time: Read<'a, TimeInfo>,
    map: Read<'a, Map>,
    trans: ReadStorage<'a, Transform>,
    itinerarys: WriteStorage<'a, Itinerary>,
}

impl<'a> System<'a> for ItinerarySystem {
    type SystemData = ItinerarySystemData<'a>;

    fn run(&mut self, mut data: Self::SystemData) {
        let time = &data.time;
        let map = &data.map;
        (&data.trans, &mut data.itinerarys)
            .par_join()
            .for_each(|(trans, it)| it.update(trans.position(), time, map));
    }
}
