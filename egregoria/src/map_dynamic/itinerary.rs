use common::GameTime;
use geom::Transform;
use geom::Vec2;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::*;
use legion::system;
use map_model::{Map, Pathfinder, Traversable, TraverseDirection, TraverseKind};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Inspect)]
pub struct Itinerary {
    kind: ItineraryKind,
    local_path: Vec<Vec2>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItineraryKind {
    None,
    WaitUntil(f64),
    Simple,
    Route(Route),
}

#[derive(Debug, Serialize, Deserialize, Inspect)]
pub struct Route {
    /// Route is reversed, allows for efficient popping
    pub reversed_route: Vec<Traversable>,
    pub end_pos: Vec2,
    pub cur: Traversable,
}

pub const OBJECTIVE_OK_DIST: f32 = 4.5;

impl Itinerary {
    pub fn none() -> Self {
        Self {
            kind: ItineraryKind::None,
            local_path: Default::default(),
        }
    }

    pub fn simple(path: Vec<Vec2>) -> Self {
        Self {
            kind: ItineraryKind::Simple,
            local_path: path,
        }
    }

    pub fn wait_until(x: f64) -> Self {
        Self {
            kind: ItineraryKind::WaitUntil(x),
            local_path: Default::default(),
        }
    }

    pub fn route(start: Vec2, end: Vec2, map: &Map, pather: &impl Pathfinder) -> Option<Itinerary> {
        let start_lane = pather.nearest_lane(map, start)?;
        let end_lane = pather.nearest_lane(map, end)?;

        if start_lane == end_lane {
            if let Some(p) = pather.local_route(map, start_lane, start, end) {
                return Some(Itinerary::simple(p.into_vec()));
            }
        }

        let mut cur = Traversable::new(TraverseKind::Lane(start_lane), TraverseDirection::Forward);

        let mut reversed_route: Vec<Traversable> =
            pather.path(map, cur, end_lane)?.into_iter().rev().collect();

        reversed_route.pop(); // Remove start

        if let Some(&Traversable {
            kind: TraverseKind::Lane(id),
            ..
        }) = reversed_route.last()
        {
            if id == start_lane {
                cur = reversed_route.pop().unwrap();
            }
        }

        let kind = ItineraryKind::Route(Route {
            reversed_route,
            end_pos: end,
            cur,
        });

        let points = cur.points(map).unwrap();
        let (proj, segid, dir) = points.project_segment_dir(start);

        let mut points = points.into_vec();
        points.drain(..segid);

        let mut it = Self {
            kind,
            local_path: points,
        };
        it.prepend_local_path([proj + dir * 3.5].iter().copied());
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

                let points = r.cur.points(map)?;
                if r.reversed_route.is_empty() {
                    let (proj_pos, id) = points.project_segment(r.end_pos);
                    self.local_path.extend(&points.as_slice()[..id]);
                    self.local_path.push(proj_pos);
                    self.local_path.push(r.end_pos);
                } else {
                    self.local_path = points.into_vec();
                }
            }
        }
        v
    }

    pub fn update(&mut self, position: Vec2, time: u32, map: &Map) {
        if let Some(p) = self.get_point() {
            let term = self.is_terminal();
            if position.is_close(p, 2.0) && term {
                self.advance(map);
            }
            if position.is_close(p, OBJECTIVE_OK_DIST) && !term {
                if self.remaining_points() > 1 {
                    self.advance(map);
                    return;
                }

                let k = unwrap_or!(self.get_travers(), {
                    *self = Itinerary::none();
                    return;
                });

                if k.can_pass(time, map.lanes()) {
                    self.advance(map);
                }
            }
        }
    }

    pub fn end_pos(&self) -> Option<Vec2> {
        match &self.kind {
            ItineraryKind::None => None,
            ItineraryKind::WaitUntil(_) => None,
            ItineraryKind::Simple => self.local_path.last().copied(),
            ItineraryKind::Route(r) => Some(r.end_pos),
        }
    }

    pub fn remaining_points(&self) -> usize {
        self.local_path.len()
    }

    pub fn is_terminal(&self) -> bool {
        match &self.kind {
            ItineraryKind::None | ItineraryKind::WaitUntil(_) => true,
            ItineraryKind::Simple => self.remaining_points() == 1,
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
            ItineraryKind::Simple => self.local_path.last().copied(),
            ItineraryKind::Route(Route { end_pos, .. }) => Some(*end_pos),
        }
    }

    pub fn get_travers(&self) -> Option<&Traversable> {
        match &self.kind {
            ItineraryKind::None | ItineraryKind::WaitUntil(_) | ItineraryKind::Simple => None,
            ItineraryKind::Route(Route { cur, .. }) => Some(cur),
        }
    }

    pub fn kind(&self) -> &ItineraryKind {
        &self.kind
    }

    pub fn local_path(&self) -> &[Vec2] {
        &self.local_path
    }

    pub fn prepend_local_path(&mut self, points: impl IntoIterator<Item = Vec2>) {
        self.local_path.splice(0..0, points.into_iter());
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

impl InspectRenderDefault<ItineraryKind> for ItineraryKind {
    fn render(_: &[&ItineraryKind], _: &'static str, _: &Ui, _: &InspectArgsDefault) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut ItineraryKind],
        label: &'static str,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!()
        }
        let d = &mut *data[0];
        use imgui::im_str;
        match d {
            ItineraryKind::None => ui.text(im_str!("None {}", label)),
            ItineraryKind::WaitUntil(time) => ui.text(im_str!("WaitUntil({}) {}", time, label)),
            ItineraryKind::Simple => ui.text(im_str!("Simple {}", label)),
            ItineraryKind::Route(r) => {
                return <Route as InspectRenderDefault<Route>>::render_mut(
                    &mut [r],
                    label,
                    ui,
                    args,
                );
            }
        };
        false
    }
}

register_system!(itinerary_update);
#[system(par_for_each)]
pub fn itinerary_update(
    #[resource] time: &GameTime,
    #[resource] map: &Map,
    trans: &Transform,
    it: &mut Itinerary,
) {
    it.update(trans.position(), time.seconds, map)
}
