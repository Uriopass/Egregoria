use crate::map::{Map, PathKind, Pathfinder, Traversable, TraverseDirection, TraverseKind};
use crate::utils::time::GameTime;
use crate::Kinematics;
use geom::{Follower, Polyline3Queue, Transform, Vec3};
use hecs::{Entity, World};
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};
use imgui_inspect_derive::Inspect;
use rayon::prelude::{ParallelBridge, ParallelIterator};
use resources::Resources;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ItineraryFollower {
    pub leader: Entity,
    pub follower: Follower,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItineraryLeader {
    pub past: Polyline3Queue,
}

#[derive(Default, Debug, Serialize, Deserialize, Inspect)]
pub struct Itinerary {
    kind: ItineraryKind,
    reversed_local_path: Vec<Vec3>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ItineraryKind {
    None,
    WaitUntil(f64),
    Simple(Vec3),
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
    pub const NONE: Self = Self {
        kind: ItineraryKind::None,
        reversed_local_path: Vec::new(),
    };

    pub fn simple(mut path: Vec<Vec3>) -> Self {
        path.reverse();
        Self {
            kind: ItineraryKind::Simple(*path.last().unwrap()),
            reversed_local_path: path,
        }
    }

    pub fn wait_until(x: f64) -> Self {
        Self {
            kind: ItineraryKind::WaitUntil(x),
            reversed_local_path: Default::default(),
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
            reversed_local_path: Default::default(),
        }
    }

    pub fn route(start: Vec3, end: Vec3, map: &Map, pathkind: PathKind) -> Option<Itinerary> {
        let start_lane = pathkind.nearest_lane(map, start)?;
        let end_lane = pathkind.nearest_lane(map, end)?;

        let mut cur = Traversable::new(TraverseKind::Lane(start_lane), TraverseDirection::Forward);

        if start_lane == end_lane {
            if let Some(mut p) = pathkind.local_route(map, start_lane, start, end) {
                p.reverse();
                return Some(Itinerary {
                    kind: ItineraryKind::Route(
                        Route {
                            reversed_route: vec![],
                            end_pos: end,
                            cur,
                        },
                        pathkind,
                    ),
                    reversed_local_path: p.into_vec(),
                });
            }
        }

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
        points.reverse();

        let mut it = Self {
            kind,
            reversed_local_path: points,
        };
        if matches!(pathkind, PathKind::Rail) {
            return Some(it);
        }

        it.prepend_local_path([proj + dir * 3.5].iter().copied());
        Some(it)
    }

    fn advance(&mut self, map: &Map) -> Option<Vec3> {
        let v = self.reversed_local_path.pop();

        if self.reversed_local_path.is_empty() {
            if let ItineraryKind::Route(ref mut r, pathkind) = self.kind {
                r.cur = r.reversed_route.pop()?;

                let mut points = match r.cur.points(map) {
                    Some(x) => x,
                    None => {
                        *self = Self::wait_for_reroute(pathkind, r.end_pos);
                        return None;
                    }
                };

                if r.reversed_route.is_empty() {
                    let (proj_pos, id) = points.project_segment(r.end_pos);
                    #[allow(clippy::indexing_slicing)]
                    self.reversed_local_path.push(r.end_pos);
                    self.reversed_local_path.push(proj_pos);
                    self.reversed_local_path
                        .extend((&points.as_slice()[..id]).iter().rev());
                } else {
                    points.reverse();
                    self.reversed_local_path = points.into_vec();
                }
            }
        }
        v
    }

    pub fn update_rail(
        &mut self,
        mut position: Vec3,
        mut dist_to_move: f32,
        time: u32,
        map: &Map,
    ) -> Vec3 {
        while let Some(p) = self.get_point() {
            let dist = position.distance(p);
            if dist <= dist_to_move + 0.01 {
                dist_to_move -= dist;
                position = p;
                if self.is_terminal() {
                    self.advance(map);
                    return p;
                }

                if self.remaining_points() > 1 {
                    self.advance(map);
                    continue;
                }

                let k = unwrap_or!(self.get_travers(), {
                    *self = Itinerary::NONE;
                    return p;
                });

                if k.can_pass(time, map.lanes()) {
                    self.advance(map);
                    continue;
                }
                return p;
            }

            return position + (p - position).normalize_to(dist_to_move);
        }

        if let ItineraryKind::WaitForReroute {
            kind,
            dest,
            ref mut wait_ticks,
        } = self.kind
        {
            if *wait_ticks > 0 {
                *wait_ticks -= 1;
                return position;
            }
            *self = unwrap_or!(Self::route(position, dest, map, kind), {
                *wait_ticks = 200;
                return position;
            });
        }
        position
    }

    pub fn end_pos(&self) -> Option<Vec3> {
        match self.kind {
            ItineraryKind::None => None,
            ItineraryKind::WaitUntil(_) | ItineraryKind::WaitForReroute { .. } => None,
            ItineraryKind::Simple(e) => Some(e),
            ItineraryKind::Route(ref r, _) => Some(r.end_pos),
        }
    }

    pub fn remaining_points(&self) -> usize {
        self.reversed_local_path.len()
    }

    pub fn is_terminal(&self) -> bool {
        match &self.kind {
            ItineraryKind::None | ItineraryKind::WaitUntil(_) => true,
            ItineraryKind::WaitForReroute { .. } => false,
            ItineraryKind::Simple(_) => self.remaining_points() <= 1,
            ItineraryKind::Route(Route { reversed_route, .. }, _) => {
                reversed_route.is_empty() && self.remaining_points() <= 1
            }
        }
    }

    pub fn get_point(&self) -> Option<Vec3> {
        self.reversed_local_path.last().copied()
    }

    pub fn get_terminal(&self) -> Option<Vec3> {
        if !self.is_terminal() {
            return None;
        }
        match self.kind {
            ItineraryKind::None
            | ItineraryKind::WaitUntil(_)
            | ItineraryKind::WaitForReroute { .. } => None,
            ItineraryKind::Simple(e) => Some(e),
            ItineraryKind::Route(Route { end_pos, .. }, _) => Some(end_pos),
        }
    }

    pub fn get_travers(&self) -> Option<&Traversable> {
        match &self.kind {
            ItineraryKind::None
            | ItineraryKind::WaitUntil(_)
            | ItineraryKind::Simple(_)
            | ItineraryKind::WaitForReroute { .. } => None,
            ItineraryKind::Route(Route { cur, .. }, _) => Some(cur),
        }
    }

    pub fn kind(&self) -> &ItineraryKind {
        &self.kind
    }

    pub fn local_path(&self) -> &[Vec3] {
        &self.reversed_local_path
    }

    pub fn prepend_local_path(&mut self, points: impl IntoIterator<Item = Vec3>) {
        self.reversed_local_path.extend(points);
    }

    pub fn has_ended(&self, time: f64) -> bool {
        match self.kind {
            ItineraryKind::WaitUntil(x) => time > x,
            ItineraryKind::WaitForReroute { .. } => false,
            _ => self.reversed_local_path.is_empty(),
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
            ItineraryKind::Simple(e) => ui.text(format!("Simple {} to {}", label, e)),
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
            ItineraryKind::Simple(e) => ui.text(format!("Simple {} to {}", label, e)),
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

#[profiling::function]
pub fn itinerary_update(world: &mut World, resources: &mut Resources) {
    let time = &*resources.get::<GameTime>().unwrap();
    let map = &*resources.get::<Map>().unwrap();
    world
        .query::<(&mut Transform, &Kinematics, &mut Itinerary)>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|chunk| {
            chunk.for_each(|(_, (trans, kin, it))| {
                trans.position =
                    it.update_rail(trans.position, kin.speed * time.delta, time.seconds, map);
            })
        });
    world
        .query::<(&Transform, &mut ItineraryLeader)>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|chunk| {
            chunk.for_each(|(_, (trans, leader))| {
                leader.past.push(trans.position);
            })
        });
    world
        .query::<(&mut Transform, &mut ItineraryFollower)>()
        .iter_batched(32)
        .par_bridge()
        .for_each(|chunk| {
            chunk.for_each(|(_, (trans, follow))| {
                let leader = unwrap_orr!(world.get::<ItineraryLeader>(follow.leader), return);
                let (pos, dir) = follow.follower.update(&leader.past);
                trans.position = pos;
                trans.dir = dir;
            })
        });
}
