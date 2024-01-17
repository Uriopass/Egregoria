use crate::map::{Map, PathKind, Pathfinder, Traversable, TraverseDirection, TraverseKind};
use crate::utils::resources::Resources;
use crate::world::TrainID;
use crate::World;
use egui_inspect::egui::Ui;
use egui_inspect::{Inspect, InspectArgs};
use geom::{Follower, Polyline3Queue, Transform, Vec3};
use prototypes::{GameTime, Tick, DELTA};
use serde::{Deserialize, Serialize};

#[derive(Inspect, Debug, Serialize, Deserialize)]
pub struct ItineraryFollower {
    pub leader: TrainID,
    #[inspect(skip)]
    pub head: Follower,
    #[inspect(skip)]
    pub tail: Follower,
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

#[derive(Debug, Default, Serialize, Deserialize)]
pub enum ItineraryKind {
    #[default]
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

    pub fn route(
        tick: Tick,
        start: Vec3,
        end: Vec3,
        map: &Map,
        pathkind: PathKind,
    ) -> Option<Itinerary> {
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
            .path(map, tick, cur, end_lane)?
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

    fn advance(&mut self, map: &Map, position: Vec3) -> Option<Vec3> {
        let v = self.reversed_local_path.pop();

        if self.reversed_local_path.is_empty() {
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
                    self.reversed_local_path = pathkind
                        .local_route(map, r.cur.destination_lane(), position, r.end_pos)
                        .unwrap_or(points)
                        .into_vec();
                } else {
                    self.reversed_local_path = points.into_vec();
                }
                self.reversed_local_path.reverse();
            }
        }
        v
    }

    pub fn update(
        &mut self,
        mut position: Vec3,
        mut dist_to_move: f32,
        tick: Tick,
        time: u32,
        map: &Map,
    ) -> Vec3 {
        while let Some(p) = self.get_point() {
            let dist = position.distance(p);
            if dist <= dist_to_move + 0.01 {
                dist_to_move -= dist;
                position = p;
                if self.is_terminal() {
                    self.advance(map, position);
                    return p;
                }

                if self.remaining_points() > 1 {
                    self.advance(map, position);
                    continue;
                }

                let k = unwrap_or!(self.get_travers(), {
                    *self = Itinerary::NONE;
                    return p;
                });

                if k.can_pass(time, map.lanes()) {
                    self.advance(map, position);
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
            *self = unwrap_or!(Self::route(tick, position, dest, map, kind), {
                *wait_ticks = 200;
                return position;
            });
        }

        position
    }

    pub fn random_route(
        rng: u64,
        position: Vec3,
        tick: Tick,
        map: &Map,
        pathkind: PathKind,
    ) -> Option<Itinerary> {
        let lanes = &map.lanes;
        let lane = lanes.values().nth(rng as usize % lanes.len())?;
        if !pathkind.authorized_lane(lane.kind) {
            return None;
        }
        Itinerary::route(
            tick,
            position,
            lane.points.point_along(lane.points.length() * 0.5),
            map,
            pathkind,
        )
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

    pub fn get_route(&self) -> Option<&Route> {
        match &self.kind {
            ItineraryKind::Route(r, _) => Some(r),
            _ => None,
        }
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
            ItineraryKind::Route(
                Route {
                    ref reversed_route, ..
                },
                _,
            ) => reversed_route.is_empty() && self.reversed_local_path.is_empty(),
            _ => self.reversed_local_path.is_empty(),
        }
    }

    pub fn is_none_or_wait(&self) -> bool {
        matches!(self.kind, ItineraryKind::None | ItineraryKind::WaitUntil(_))
    }

    pub fn is_simple(&self) -> bool {
        matches!(self.kind, ItineraryKind::Simple(_))
    }
}

impl Inspect<ItineraryKind> for ItineraryKind {
    fn render(d: &ItineraryKind, label: &'static str, ui: &mut Ui, args: &InspectArgs) {
        match *d {
            ItineraryKind::None => {
                ui.label(format!("None {label}"));
            }
            ItineraryKind::WaitUntil(time) => {
                ui.label(format!("WaitUntil({time}) {label}"));
            }
            ItineraryKind::Simple(e) => {
                ui.label(format!("Simple {label} to {e}"));
            }
            ItineraryKind::Route(ref r, _) => {
                <Route as Inspect<Route>>::render(r, label, ui, args);
            }
            ItineraryKind::WaitForReroute { wait_ticks, .. } => {
                ui.label(format!("wait for reroute: {wait_ticks}"));
            }
        };
    }

    fn render_mut(
        d: &mut ItineraryKind,
        label: &'static str,
        ui: &mut Ui,
        args: &InspectArgs,
    ) -> bool {
        match *d {
            ItineraryKind::None => {
                ui.label(format!("None {label}"));
            }
            ItineraryKind::WaitUntil(time) => {
                ui.label(format!("WaitUntil({time}) {label}"));
            }
            ItineraryKind::Simple(e) => {
                ui.label(format!("Simple {label} to {e}"));
            }
            ItineraryKind::Route(ref mut r, _) => {
                return <Route as Inspect<Route>>::render_mut(r, label, ui, args);
            }
            ItineraryKind::WaitForReroute { wait_ticks, .. } => {
                ui.label(format!("wait for reroute: {wait_ticks}"));
            }
        };
        false
    }
}

pub fn itinerary_update(world: &mut World, resources: &mut Resources) {
    profiling::scope!("map_dynamic::itinerary_update");
    let time = &*resources.read::<GameTime>();
    let map = &*resources.read::<Map>();
    let tick = resources.read::<GameTime>().tick;

    world.query_it_trans_speed().for_each(
        |(it, trans, speed): (&mut Itinerary, &mut Transform, f32)| {
            trans.pos = it.update(trans.pos, speed * DELTA, tick, time.seconds, map);
        },
    );

    world.trains.values_mut().for_each(|train| {
        train.leader.past.push(train.trans.pos);
    });

    world.wagons.values_mut().for_each(|wagon| {
        let leader = &unwrap_ret!(world.trains.get(wagon.itfollower.leader)).leader;
        let (pos, dir) = wagon.itfollower.head.update(&leader.past);
        let (pos2, dir2) = wagon.itfollower.tail.update(&leader.past);
        wagon.trans.pos = (pos + pos2) * 0.5;
        wagon.trans.dir = (dir + dir2).try_normalize().unwrap_or(dir);
    });
}
