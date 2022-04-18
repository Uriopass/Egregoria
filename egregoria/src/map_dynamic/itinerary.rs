use crate::utils::time::GameTime;
use geom::{Transform, Vec3};
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::Inspect;
use legion::world::SubWorld;
use legion::{system, Query};
use map_model::{Map, PathKind, Pathfinder, Traversable, TraverseDirection, TraverseKind};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Inspect)]
pub struct Itinerary {
    kind: ItineraryKind,
    // fixme: replace local path with newtype stack to be popped in O(1)
    local_path: Vec<Vec3>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItineraryKind {
    None,
    WaitUntil(f64),
    Simple,
    Route(Route, PathKind),
    WaitForReroute {
        kind: PathKind,
        dest: Vec3,
        wait_ticks: u16,
    },
}

#[derive(Debug, Serialize, Deserialize, Inspect)]
pub struct Route {
    /// Route is reversed, allows for efficient popping
    pub reversed_route: Vec<Traversable>,
    pub end_pos: Vec3,
    pub cur: Traversable,
}

pub const OBJECTIVE_OK_DIST: f32 = 3.0;

impl Itinerary {
    pub fn none() -> Self {
        Self {
            kind: ItineraryKind::None,
            local_path: Default::default(),
        }
    }

    pub fn simple(path: Vec<Vec3>) -> Self {
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

    pub fn is_wait_for_reroute(&self) -> Option<u16> {
        if let ItineraryKind::WaitForReroute { wait_ticks, .. } = self.kind {
            Some(wait_ticks)
        } else {
            None
        }
    }

    pub fn wait_for_reroute(kind: PathKind, dest: Vec3) -> Self {
        Self {
            kind: ItineraryKind::WaitForReroute {
                kind,
                dest,
                wait_ticks: 0,
            },
            local_path: Default::default(),
        }
    }

    pub fn route(start: Vec3, end: Vec3, map: &Map, pathkind: PathKind) -> Option<Itinerary> {
        let start_lane = pathkind.nearest_lane(map, start)?;
        let end_lane = pathkind.nearest_lane(map, end)?;

        if start_lane == end_lane {
            if let Some(p) = pathkind.local_route(map, start_lane, start, end) {
                return Some(Itinerary::simple(p.into_vec()));
            }
        }

        let mut cur = Traversable::new(TraverseKind::Lane(start_lane), TraverseDirection::Forward);

        let mut reversed_route: Vec<Traversable> = pathkind
            .path(map, cur, end_lane)?
            .into_iter()
            .rev()
            .collect();

        reversed_route.pop(); // Remove start

        if let Some(&Traversable {
            kind: TraverseKind::Lane(id),
            ..
        }) = reversed_route.last()
        {
            #[allow(clippy::unwrap_used)] // just checked that last is some
            if id == start_lane {
                cur = reversed_route.pop().unwrap();
            }
        }

        let kind = ItineraryKind::Route(
            Route {
                reversed_route,
                end_pos: end,
                cur,
            },
            pathkind,
        );

        let points = cur.points(map)?;
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

    fn advance(&mut self, map: &Map) -> Option<Vec3> {
        let v = if self.local_path.is_empty() {
            None
        } else {
            Some(self.local_path.remove(0))
        };

        if self.local_path.is_empty() {
            if let ItineraryKind::Route(ref mut r, pathkind) = self.kind {
                r.cur = r.reversed_route.pop()?;

                let points = match r.cur.points(map) {
                    Some(x) => x,
                    None => {
                        *self = Self::wait_for_reroute(pathkind, r.end_pos);
                        return None;
                    }
                };

                if r.reversed_route.is_empty() {
                    let (proj_pos, id) = points.project_segment(r.end_pos);
                    #[allow(clippy::indexing_slicing)]
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

    #[allow(clippy::collapsible_else_if)]
    pub fn update(&mut self, position: Vec3, time: u32, map: &Map) {
        if let Some(p) = self.get_point() {
            if self.is_terminal() {
                if position.is_close(p, 1.5) {
                    self.advance(map);
                }
            } else {
                if position.is_close(p, OBJECTIVE_OK_DIST) {
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
            return;
        }

        if let ItineraryKind::WaitForReroute {
            kind,
            dest,
            ref mut wait_ticks,
        } = self.kind
        {
            if *wait_ticks > 0 {
                *wait_ticks -= 1;
                return;
            }
            *self = unwrap_or!(Self::route(position, dest, map, kind), {
                *wait_ticks = 200;
                return;
            });
        }
    }

    pub fn end_pos(&self) -> Option<Vec3> {
        match &self.kind {
            ItineraryKind::None => None,
            ItineraryKind::WaitUntil(_) | ItineraryKind::WaitForReroute { .. } => None,
            ItineraryKind::Simple => self.local_path.last().copied(),
            ItineraryKind::Route(r, _) => Some(r.end_pos),
        }
    }

    pub fn remaining_points(&self) -> usize {
        self.local_path.len()
    }

    pub fn is_terminal(&self) -> bool {
        match &self.kind {
            ItineraryKind::None | ItineraryKind::WaitUntil(_) => true,
            ItineraryKind::WaitForReroute { .. } => false,
            ItineraryKind::Simple => self.remaining_points() == 1,
            ItineraryKind::Route(Route { reversed_route, .. }, _) => {
                reversed_route.is_empty() && self.remaining_points() == 1
            }
        }
    }

    pub fn get_point(&self) -> Option<Vec3> {
        self.local_path.first().copied()
    }

    pub fn get_terminal(&self) -> Option<Vec3> {
        if !self.is_terminal() {
            return None;
        }
        match &self.kind {
            ItineraryKind::None
            | ItineraryKind::WaitUntil(_)
            | ItineraryKind::WaitForReroute { .. } => None,
            ItineraryKind::Simple => self.local_path.last().copied(),
            ItineraryKind::Route(Route { end_pos, .. }, _) => Some(*end_pos),
        }
    }

    pub fn get_travers(&self) -> Option<&Traversable> {
        match &self.kind {
            ItineraryKind::None
            | ItineraryKind::WaitUntil(_)
            | ItineraryKind::Simple
            | ItineraryKind::WaitForReroute { .. } => None,
            ItineraryKind::Route(Route { cur, .. }, _) => Some(cur),
        }
    }

    pub fn kind(&self) -> &ItineraryKind {
        &self.kind
    }

    pub fn local_path(&self) -> &[Vec3] {
        &self.local_path
    }

    pub fn prepend_local_path(&mut self, points: impl IntoIterator<Item = Vec3>) {
        self.local_path.splice(0..0, points.into_iter());
    }

    pub fn has_ended(&self, time: f64) -> bool {
        match self.kind {
            ItineraryKind::WaitUntil(x) => time > x,
            ItineraryKind::WaitForReroute { .. } => false,
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
    fn render(
        data: &[&ItineraryKind],
        label: &'static str,
        ui: &Ui<'_>,
        args: &InspectArgsDefault,
    ) {
        let d = *unwrap_ret!(data.get(0));

        match d {
            ItineraryKind::None => ui.text(format!("None {}", label)),
            ItineraryKind::WaitUntil(time) => ui.text(format!("WaitUntil({}) {}", time, label)),
            ItineraryKind::Simple => ui.text(format!("Simple {}", label)),
            ItineraryKind::Route(r, _) => {
                <Route as InspectRenderDefault<Route>>::render(&[r], label, ui, args);
            }
            ItineraryKind::WaitForReroute { wait_ticks, .. } => {
                ui.text(format!("wait for reroute: {}", *wait_ticks));
            }
        };
    }

    fn render_mut(
        data: &mut [&mut ItineraryKind],
        label: &'static str,
        ui: &Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        let d = &mut *unwrap_ret!(data.get_mut(0), false);

        match d {
            ItineraryKind::None => ui.text(format!("None {}", label)),
            ItineraryKind::WaitUntil(time) => ui.text(format!("WaitUntil({}) {}", time, label)),
            ItineraryKind::Simple => ui.text(format!("Simple {}", label)),
            ItineraryKind::Route(r, _) => {
                return <Route as InspectRenderDefault<Route>>::render_mut(
                    &mut [r],
                    label,
                    ui,
                    args,
                );
            }
            ItineraryKind::WaitForReroute { wait_ticks, .. } => {
                ui.text(format!("wait for reroute: {}", *wait_ticks));
            }
        };
        false
    }
}

type Qry<'a, 'b> = (&'a Transform, &'b mut Itinerary);
register_system!(itinerary_update);
#[system]
pub fn itinerary_update(
    #[resource] time: &GameTime,
    #[resource] map: &Map,
    qry: &mut Query<Qry<'_, '_>>,
    world: &mut SubWorld<'_>,
) {
    qry.par_for_each_mut(world, |(trans, it): Qry<'_, '_>| {
        it.update(trans.position, time.seconds, map)
    });
}
