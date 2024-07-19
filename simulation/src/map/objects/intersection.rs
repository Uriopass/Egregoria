use crate::map::{
    Intersections, LaneID, LaneKind, Lanes, LightPolicy, Road, RoadID, Roads, SpatialMap,
    TraverseDirection, Turn, TurnID, TurnPolicy,
};
use geom::{pseudo_angle, Circle, Ray};
use geom::{Vec2, Vec3};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;
use std::collections::BTreeSet;

new_key_type! {
    pub struct IntersectionID;
}

impl IntersectionID {
    pub fn as_ffi(self) -> u64 {
        self.0.as_ffi()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    pub pos: Vec3,
    pub radius: f32,

    turns: BTreeSet<Turn>,

    // sorted by angle
    pub roads: Vec<RoadID>,

    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

impl Intersection {
    pub fn make(store: &mut Intersections, spatial: &mut SpatialMap, pos: Vec3) -> IntersectionID {
        let id = store.insert_with_key(|id| Intersection {
            id,
            pos,
            radius: 0.0,
            turns: Default::default(),
            roads: Default::default(),
            turn_policy: Default::default(),
            light_policy: Default::default(),
        });
        spatial.insert(&store[id]);
        id
    }

    pub fn add_road(&mut self, roads: &Roads, road: &Road) {
        self.roads.push(road.id);

        let id = self.id;
        self.roads.retain(|&id| roads.contains_key(id));
        self.roads.sort_by_key(|&road| {
            #[allow(clippy::indexing_slicing)]
            OrderedFloat(pseudo_angle(roads[road].dir_from(id)))
        });
    }

    pub fn bcircle(&self) -> Circle {
        Circle {
            center: self.pos.xy(),
            radius: self.radius,
        }
    }

    fn update_radius(&mut self, roads: &Roads) {
        self.radius = self
            .roads
            .iter()
            .flat_map(|x| roads.get(*x))
            .map(|x| {
                if self.is_roundabout() {
                    OrderedFloat(x.interface_from(self.id))
                } else {
                    OrderedFloat(x.width)
                }
            })
            .max()
            .map(|x| x.0)
            .unwrap_or(10.0);
    }

    pub fn remove_road(&mut self, road_id: RoadID) {
        self.roads.retain(|x| *x != road_id);
    }

    pub fn update_turns(&mut self, lanes: &Lanes, roads: &Roads) {
        self.turns = self
            .turn_policy
            .generate_turns(self, lanes, roads)
            .into_iter()
            .map(|(id, kind)| Turn::new(id, kind))
            .collect();

        self.turns = std::mem::take(&mut self.turns)
            .into_iter()
            .map(|mut x| {
                x.make_points(lanes, self);
                x
            })
            .collect();
    }

    pub fn update_traffic_control(&self, lanes: &mut Lanes, roads: &Roads) {
        self.light_policy.apply(self, lanes, roads);
    }

    const MIN_INTERFACE: f32 = 9.0;
    pub fn update_interface_radius(&mut self, roads: &mut Roads) {
        let id = self.id;

        match *self.roads {
            [] => return,
            [r1_id] => {
                let r = &mut roads[r1_id];
                r.set_interface(id, Self::empty_interface(r.width));
                return;
            }
            [r1_id, r2_id] => {
                let (r1, r2) = (&roads[r1_id], &roads[r2_id]);
                let (dir1, dir2) = (r1.dir_from(id), r2.dir_from(id));
                let (r1w, r2w) = (r1.width, r2.width);
                let elbow = (dir1 + dir2) * 0.5;

                if elbow.mag() < 0.001 {
                    roads[r1_id].set_interface(id, 1.0);
                    roads[r2_id].set_interface(id, 1.0);
                    return;
                }

                let ray1 = Ray::new(
                    self.pos.xy()
                        + dir1.perpendicular()
                            * dir1.perpendicular().dot(elbow).signum()
                            * r1w
                            * 0.5,
                    dir1,
                );
                let ray2 = Ray::new(
                    self.pos.xy()
                        + dir2.perpendicular()
                            * dir2.perpendicular().dot(elbow).signum()
                            * r2w
                            * 0.5,
                    dir2,
                );

                let Some((dist_a, dist_b)) = ray1.both_dist_to_inter(&ray2) else {
                    roads[r1_id].set_interface(id, Self::empty_interface(r1w));
                    roads[r2_id].set_interface(id, Self::empty_interface(r2w));
                    return;
                };

                roads[r1_id].set_interface(id, dist_a);
                roads[r2_id].set_interface(id, dist_b);
                return;
            }
            _ => {}
        }

        for &r in &self.roads {
            let r = &mut roads[r];
            r.set_interface(id, Self::empty_interface(r.width));
        }

        if self.is_roundabout() {
            if let Some(rb) = self.turn_policy.roundabout {
                for &r in &self.roads {
                    let r = &mut roads[r];
                    r.max_interface(id, rb.radius * 1.1 + 5.0);
                }
            }
        }

        for i in 0..self.roads.len() {
            let r1_id = self.roads[i];
            let r2_id = self.roads[(i + 1) % self.roads.len()];

            let (r1, r2) = (&roads[r1_id], &roads[r2_id]);
            let (dir1, dir2) = (r1.dir_from(id), r2.dir_from(id));

            let min_dist = if dir1.angle(dir2).abs() < 0.17453292 {
                self.interface_calc_numerically(r1.width, r2.width, r1, r2)
            } else {
                Self::interface_calc_formula(r1.width, r2.width, dir1, dir2)
            };

            roads[r1_id].max_interface(id, min_dist);
            roads[r2_id].max_interface(id, min_dist);
        }

        self.update_radius(roads);
    }

    fn interface_calc_formula(w1: f32, w2: f32, dir1: Vec2, dir2: Vec2) -> f32 {
        let hwidth1 = w1 * 0.5;
        let hwidth2 = w2 * 0.5;

        let w = hwidth1.hypot(hwidth2);

        let d = dir1.dot(dir2).clamp(0.0, 1.0);
        let sin = (1.0 - d * d).sqrt();

        (w * 1.1 / sin).min(50.0)
    }

    fn interface_calc_numerically(&self, w1: f32, w2: f32, r1: &Road, r2: &Road) -> f32 {
        let w: f32 = (w1 + w2) * 0.80;

        let mut points1: Vec<(Vec3, Vec3)> = r1
            .points()
            .points_dirs_along((1..r1.points().length() as i32).map(|d| d as f32))
            .collect();
        let mut points2: Vec<(Vec3, Vec3)> = r2
            .points()
            .points_dirs_along((1..r2.points().length() as i32).map(|d| d as f32))
            .collect();

        if r1.src != self.id {
            points1.reverse();
        }
        if r2.src != self.id {
            points2.reverse();
        }

        points1
            .into_iter()
            .zip(points2)
            .map(|((p1, _), (p2, _))| (p1.xy(), p2.xy()))
            .find(|p| p.0.distance(p.1) > w)
            .map(|p| (self.pos.xy().distance(p.0) + self.pos.xy().distance(p.0)) * 0.5)
            .unwrap_or(50.0)
    }

    pub fn empty_interface(width: f32) -> f32 {
        (width * 0.8).max(Self::MIN_INTERFACE)
    }

    pub fn interface_at(&self, roads: &Roads, width: f32, dir: Vec2) -> f32 {
        let mut max_inter = Self::empty_interface(width);
        let id = self.id;
        for &r1_id in &self.roads {
            let r1 = unwrap_cont!(roads.get(r1_id));
            max_inter = max_inter.max(Self::interface_calc_formula(
                r1.width,
                width,
                r1.dir_from(id),
                dir,
            ));
        }
        max_inter
    }

    pub fn is_roundabout(&self) -> bool {
        self.turn_policy.roundabout.is_some() && self.roads.len() > 1
    }

    pub fn undirected_neighbors<'a>(
        &'a self,
        roads: &'a Roads,
    ) -> impl Iterator<Item = IntersectionID> + 'a {
        self.roads
            .iter()
            .flat_map(move |&x| roads.get(x).and_then(|r| r.other_end(self.id)))
    }

    pub fn vehicle_neighbours<'a>(
        &'a self,
        roads: &'a Roads,
    ) -> impl Iterator<Item = IntersectionID> + 'a {
        let id = self.id;
        self.roads.iter().flat_map(move |&x| {
            let r = roads.get(x)?;
            r.outgoing_lanes_from(id).iter().find(|(_, kind)| {
                matches!(kind, LaneKind::Driving | LaneKind::Rail | LaneKind::Bus)
            })?;
            r.other_end(id)
        })
    }

    pub fn find_turn(&self, needle: TurnID) -> Option<&Turn> {
        self.turns.get(&needle)
    }

    pub fn turns_from(
        &self,
        lane: LaneID,
    ) -> impl Iterator<Item = (TurnID, TraverseDirection)> + '_ {
        self.turns.iter().filter_map(move |Turn { id, .. }| {
            if id.src == lane {
                Some((*id, TraverseDirection::Forward))
            } else if id.bidirectional && id.dst == lane {
                Some((*id, TraverseDirection::Backward))
            } else {
                None
            }
        })
    }

    pub fn turns_to(&self, lane: LaneID) -> impl Iterator<Item = (TurnID, TraverseDirection)> + '_ {
        self.turns.iter().filter_map(move |Turn { id, .. }| {
            if id.dst == lane {
                Some((*id, TraverseDirection::Forward))
            } else if id.bidirectional && id.src == lane {
                Some((*id, TraverseDirection::Backward))
            } else {
                None
            }
        })
    }

    pub fn turns(&self) -> impl ExactSizeIterator<Item = &Turn> {
        self.turns.iter()
    }
}

debug_inspect_impl!(IntersectionID);
