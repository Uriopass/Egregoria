use crate::geometry::Vec2;
use crate::map_model::{
    Intersection, IntersectionID, Lane, LaneID, LaneKind, LanePattern, Road, RoadID, Traversable,
    TraverseKind,
};
use crate::utils::{rand_det, Choose};
use cgmath::{MetricSpace, Zero};
use ordered_float::{NotNan, OrderedFloat};
use serde::{Deserialize, Serialize};
use slotmap::DenseSlotMap;

pub type Roads = DenseSlotMap<RoadID, Road>;
pub type Lanes = DenseSlotMap<LaneID, Lane>;
pub type Intersections = DenseSlotMap<IntersectionID, Intersection>;

#[derive(Debug, Clone, Copy)]
pub enum ProjectKind {
    Inter(IntersectionID),
    Road(RoadID),
}

#[derive(Debug, Clone, Copy)]
pub struct MapProject {
    pub pos: Vec2,
    pub kind: ProjectKind,
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    roads: Roads,
    lanes: Lanes,
    intersections: Intersections,
}

impl Default for Map {
    fn default() -> Self {
        Self::empty()
    }
}

impl Map {
    pub fn empty() -> Self {
        Self {
            roads: Roads::with_key(),
            lanes: Lanes::with_key(),
            intersections: Intersections::with_key(),
        }
    }

    pub fn update_intersection(
        &mut self,
        id: IntersectionID,
        f: impl Fn(&mut Intersection) -> (),
    ) -> &Intersection {
        let inter = &mut self.intersections[id];
        f(inter);

        for x in self.intersections[id].roads.clone() {
            let other_end = self.roads[x].other_end(id);
            self.invalidate(other_end);
        }

        self.invalidate(id);
        &self.intersections[id]
    }

    fn invalidate(&mut self, id: IntersectionID) {
        let inter = &mut self.intersections[id];
        inter.update_interface_radius(&self.lanes, &self.roads);

        for x in inter.roads.clone() {
            let road = &mut self.roads[x];
            road.gen_pos(&self.intersections, &mut self.lanes);
        }

        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);
        inter.update_barycenter(&self.lanes, &self.roads);
    }

    pub fn add_intersection(&mut self, pos: Vec2) -> IntersectionID {
        Intersection::make(&mut self.intersections, pos)
    }

    pub fn remove_intersection(&mut self, src: IntersectionID) {
        for road in self.intersections[src].roads.clone() {
            self.remove_road(road);
        }

        self.intersections.remove(src);
    }

    pub fn connect(
        &mut self,
        src: IntersectionID,
        dst: IntersectionID,
        pattern: LanePattern,
    ) -> RoadID {
        let road_id = Road::make(
            &mut self.roads,
            &self.intersections,
            src,
            dst,
            &mut self.lanes,
            pattern,
        );

        let inters = &mut self.intersections;

        inters[src].add_road(road_id, &mut self.lanes, &self.roads);
        inters[dst].add_road(road_id, &mut self.lanes, &self.roads);

        self.invalidate(src);
        self.invalidate(dst);

        road_id
    }

    pub fn remove_road(&mut self, road_id: RoadID) -> Road {
        let road = self.roads.remove(road_id).unwrap();
        for lane_id in road.lanes_iter() {
            self.lanes.remove(*lane_id).unwrap();
        }

        self.intersections[road.src].remove_road(road_id, &mut self.lanes, &self.roads);
        self.intersections[road.dst].remove_road(road_id, &mut self.lanes, &self.roads);

        self.invalidate(road.src);
        self.invalidate(road.dst);
        road
    }

    pub fn clear(&mut self) {
        self.intersections.clear();
        self.lanes.clear();
        self.roads.clear();
    }

    pub fn path(&self, start: Traversable, end: LaneID) -> Option<Vec<Traversable>> {
        let inters = &self.intersections;
        let lanes = &self.lanes;

        let end_pos = inters[lanes[end].dst].pos;

        let heuristic = |t: &Traversable| {
            let pos = inters[t.destination_intersection(lanes)].pos;

            NotNan::new(pos.distance(end_pos) * 1.2).unwrap() // Inexact but (much) faster
        };

        let successors = |t: &Traversable| {
            let inter = &inters[t.destination_intersection(lanes)];
            let lane_from_id = t.destination_lane();
            let lane_from = &lanes[lane_from_id];

            let lane_travers = (
                Traversable::new(
                    TraverseKind::Lane(lane_from_id),
                    lane_from.dir_from(inter.id),
                ),
                NotNan::new(lane_from.parent_length).unwrap(),
            );

            inter
                .turns_from(lane_from_id)
                .map(|(x, dir)| (Traversable::new(TraverseKind::Turn(x), dir), NotNan::zero()))
                .chain(std::iter::once(lane_travers))
        };

        let has_arrived = |p: &Traversable| match p.kind {
            TraverseKind::Lane(id) => id == end,
            TraverseKind::Turn(_) => false,
        };

        pathfinding::directed::astar::astar(&start, successors, heuristic, has_arrived)
            .map(|(v, _)| v)
    }

    pub fn project(&self, pos: Vec2) -> Option<MapProject> {
        const THRESHOLD: f32 = 20.0;

        let (min_inter, d) = self
            .intersections()
            .values()
            .map(|inter| {
                let mut d = inter.pos.distance2(pos);
                if d > inter.interface_radius.powi(2) {
                    d = std::f32::INFINITY;
                }
                (inter, d)
            })
            .min_by_key(|(_, d)| OrderedFloat(*d))?;

        if d.is_finite() {
            return Some(MapProject {
                pos: min_inter.barycenter,
                kind: ProjectKind::Inter(min_inter.id),
            });
        }

        let (min_road, d, projected) = self
            .roads()
            .values()
            .map(|road| {
                let proj = road.interpolation_points.project(pos).unwrap();
                (road, proj.distance2(pos), proj)
            })
            .min_by_key(|(_, d, _)| OrderedFloat(*d))?;

        if d < THRESHOLD * THRESHOLD {
            let r1 = self.intersections[min_road.src].interface_radius;
            let r2 = self.intersections[min_road.dst].interface_radius;

            if projected.distance2(min_road.interpolation_points[0]) < r1 {
                return None;
            }

            if projected.distance2(min_road.interpolation_points.last().unwrap()) < r2 {
                return None;
            }

            Some(MapProject {
                pos: projected,
                kind: ProjectKind::Road(min_road.id),
            })
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.roads.is_empty() && self.lanes.is_empty() && self.intersections.is_empty()
    }

    pub fn roads(&self) -> &Roads {
        &self.roads
    }
    pub fn lanes(&self) -> &Lanes {
        &self.lanes
    }
    pub fn intersections(&self) -> &Intersections {
        &self.intersections
    }

    pub fn get_random_lane(&self, kind: LaneKind) -> Option<&Lane> {
        let l = self.roads.len();
        if l == 0 {
            return None;
        }
        let r = (rand_det::<f32>() * l as f32) as usize;

        let (_, road) = self.roads.iter().nth(r).unwrap();
        let lanes = road
            .lanes_iter()
            .filter(|x| self.lanes[**x].kind == kind)
            .collect::<Vec<&LaneID>>();

        lanes.choose().map(|x| &self.lanes[**x])
    }

    pub fn find_road(&self, a: IntersectionID, b: IntersectionID) -> Option<RoadID> {
        for r in &self.intersections[a].roads {
            let road = &self.roads[*r];
            if road.src == a && road.dst == b || (road.dst == a && road.src == b) {
                return Some(road.id);
            }
        }
        None
    }

    pub fn closest_lane(&self, p: Vec2, kind: LaneKind) -> Option<LaneID> {
        self.lanes
            .iter()
            .filter(|(_, x)| x.kind == kind)
            .min_by_key(|(_, lane)| OrderedFloat(lane.dist2_to(p)))
            .map(|(id, _)| id)
    }

    pub fn closest_inter(&self, p: Vec2) -> Option<IntersectionID> {
        self.intersections
            .iter()
            .min_by_key(|(_, inter)| OrderedFloat(inter.barycenter.distance2(p)))
            .map(|(id, _)| id)
    }

    pub fn is_neigh(&self, src: IntersectionID, dst: IntersectionID) -> bool {
        self.find_road(src, dst).is_some()
    }
}
