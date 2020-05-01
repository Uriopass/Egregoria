use crate::geometry::Vec2;
use crate::map_model::{
    Intersection, IntersectionID, Lane, LaneID, LaneKind, LanePattern, Road, RoadID,
};
use crate::utils::rand_det;
use cgmath::MetricSpace;
use ordered_float::OrderedFloat;
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
        inter.update_optimal_radius(&self.lanes, &self.roads);

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

        self.intersections[src].add_road(road_id, &mut self.lanes, &self.roads);
        self.intersections[dst].add_road(road_id, &mut self.lanes, &self.roads);

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

    /* Helpers */

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

        if lanes.is_empty() {
            return None;
        }
        let r = (rand_det::<f32>() * lanes.len() as f32) as usize;

        Some(&self.lanes[*lanes[r]])
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

    pub fn closest_lane(&self, p: Vec2) -> Option<LaneID> {
        let mut min_dist = std::f32::MAX;
        let mut closest = None;

        for (id, lane) in &self.lanes {
            let dist = lane.dist_to(p);
            if dist < min_dist {
                min_dist = dist;
                closest = Some(id);
            }
        }
        closest
    }

    pub fn is_neigh(&self, src: IntersectionID, dst: IntersectionID) -> bool {
        self.find_road(src, dst).is_some()
    }
}
