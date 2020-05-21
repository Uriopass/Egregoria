use crate::geometry::Vec2;
use crate::map_model::{
    Intersection, IntersectionID, Lane, LaneID, LaneKind, LanePattern, Road, RoadID,
};
use crate::utils::{rand_det, Choose};
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
    pub dirty: bool,
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
            dirty: true,
        }
    }

    pub fn update_intersection(&mut self, id: IntersectionID, f: impl Fn(&mut Intersection) -> ()) {
        let inter = unwrap_or!(self.intersections.get_mut(id), return);
        f(inter);

        for x in self.intersections[id].roads.clone() {
            let other_end = self.roads[x].other_end(id);
            self.invalidate(other_end);
        }

        self.invalidate(id);
    }

    fn invalidate(&mut self, id: IntersectionID) {
        self.dirty = true;
        let inter = &mut self.intersections[id];
        inter.update_interface_radius(&mut self.roads);

        for x in inter.roads.clone() {
            let road = &mut self.roads[x];
            road.gen_pos(&self.intersections, &mut self.lanes);
            self.intersections[road.other_end(id)].update_turns(&self.lanes, &self.roads);
        }

        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);
        inter.update_barycenter(&self.roads);
        inter.update_polygon(&self.roads);
    }

    pub fn add_intersection(&mut self, pos: Vec2) -> IntersectionID {
        self.dirty = true;
        Intersection::make(&mut self.intersections, pos)
    }

    pub fn remove_intersection(&mut self, src: IntersectionID) {
        self.dirty = true;
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
        self.dirty = true;
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
        self.dirty = true;
        let road = self.roads.remove(road_id).unwrap();
        for &lane_id in road.lanes_iter() {
            self.lanes.remove(lane_id);
        }

        self.intersections[road.src].remove_road(road_id, &mut self.lanes, &self.roads);
        self.intersections[road.dst].remove_road(road_id, &mut self.lanes, &self.roads);

        self.invalidate(road.src);
        self.invalidate(road.dst);
        road
    }

    pub fn clear(&mut self) {
        self.dirty = true;
        self.intersections.clear();
        self.lanes.clear();
        self.roads.clear();
    }

    pub fn project(&self, pos: Vec2) -> Option<MapProject> {
        const THRESHOLD: f32 = 15.0;

        if let Some(v) = self
            .intersections
            .values()
            .filter(|x| x.roads.is_empty())
            .min_by_key(|x| OrderedFloat(x.pos.distance2(pos)))
        {
            if v.pos.distance(pos) < 5.0 {
                return Some(MapProject {
                    pos: v.pos,
                    kind: ProjectKind::Inter(v.id),
                });
            }
        }

        let (min_road, d, projected) = self
            .roads()
            .values()
            .map(|road| {
                let proj = road.interpolation_points().project(pos).unwrap();
                (road, proj.distance2(pos), proj)
            })
            .min_by_key(|(_, d, _)| OrderedFloat(*d))?;

        if d < THRESHOLD * THRESHOLD {
            if projected.distance(min_road.src_point()) < min_road.src_interface {
                return Some(MapProject {
                    pos: self.intersections[min_road.src].barycenter,
                    kind: ProjectKind::Inter(min_road.src),
                });
            }

            if projected.distance(min_road.dst_point()) < min_road.dst_interface {
                return Some(MapProject {
                    pos: self.intersections[min_road.dst].barycenter,
                    kind: ProjectKind::Inter(min_road.dst),
                });
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
