use crate::geometry::splines::Spline;
use crate::geometry::Vec2;
use crate::map_model::{
    Intersection, IntersectionID, Lane, LaneID, LaneKind, LanePattern, Road, RoadID,
    RoadSegmentKind,
};
use ordered_float::OrderedFloat;
use rand::prelude::IteratorRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use slotmap::DenseSlotMap;

pub type Roads = DenseSlotMap<RoadID, Road>;
pub type Lanes = DenseSlotMap<LaneID, Lane>;
pub type Intersections = DenseSlotMap<IntersectionID, Intersection>;

#[derive(Debug, Clone, Copy)]
pub enum ProjectKind {
    Inter(IntersectionID),
    Road(RoadID),
    Ground,
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

        self.invalidate(id);

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
            let other_end = &mut self.intersections[self.roads[x].other_end(id)];
            other_end.update_interface_radius(&mut self.roads);

            let road = &mut self.roads[x];
            road.gen_pos(&self.intersections, &mut self.lanes);

            let other_end = &mut self.intersections[self.roads[x].other_end(id)];
            other_end.update_polygon(&self.roads);
        }

        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);
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

    pub fn split_road(&mut self, id: RoadID, pos: Vec2) -> IntersectionID {
        let r = self.remove_road(id);
        let id = self.add_intersection(pos);

        match r.segment {
            RoadSegmentKind::Straight => {
                self.connect(r.src, id, r.lane_pattern.clone(), RoadSegmentKind::Straight);
                self.connect(id, r.dst, r.lane_pattern, RoadSegmentKind::Straight);
            }
            RoadSegmentKind::Curved((from_derivative, to_derivative)) => {
                let s = Spline {
                    from: r.src_point,
                    to: r.dst_point,
                    from_derivative,
                    to_derivative,
                };
                let t_approx = s.project_t(pos, 1.0);

                let (s_from, s_to) = s.split_at(t_approx);

                self.connect(
                    r.src,
                    id,
                    r.lane_pattern.clone(),
                    RoadSegmentKind::Curved((s_from.from_derivative, s_from.to_derivative)),
                );
                self.connect(
                    id,
                    r.dst,
                    r.lane_pattern,
                    RoadSegmentKind::Curved((s_to.from_derivative, s_to.to_derivative)),
                );
            }
        }

        id
    }

    // todo: remove in favor of connect(..., RoadSegmentKind::Straight)
    pub fn connect_straight(
        &mut self,
        src: IntersectionID,
        dst: IntersectionID,
        pattern: LanePattern,
    ) -> RoadID {
        self.connect(src, dst, pattern, RoadSegmentKind::Straight)
    }

    pub fn connect(
        &mut self,
        src: IntersectionID,
        dst: IntersectionID,
        pattern: LanePattern,
        segment: RoadSegmentKind,
    ) -> RoadID {
        self.dirty = true;
        let road_id = Road::make(
            src,
            dst,
            segment,
            pattern,
            &self.intersections,
            &mut self.lanes,
            &mut self.roads,
        );

        let inters = &mut self.intersections;

        inters[src].add_road(road_id, &self.roads);
        inters[dst].add_road(road_id, &self.roads);

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

        self.intersections[road.src].remove_road(road_id);
        self.intersections[road.dst].remove_road(road_id);

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

    pub fn project(&self, pos: Vec2) -> MapProject {
        const THRESHOLD: f32 = 15.0;

        if let Some(v) = self
            .intersections
            .values()
            .filter(|x| x.roads.is_empty())
            .min_by_key(|x| OrderedFloat(x.pos.distance2(pos)))
        {
            if v.pos.distance(pos) < 5.0 {
                return MapProject {
                    pos: v.pos,
                    kind: ProjectKind::Inter(v.id),
                };
            }
        }

        let (min_road, d, projected) = match self
            .roads()
            .values()
            .map(|road| {
                let proj = road.project(pos);
                (road, proj.distance2(pos), proj)
            })
            .min_by_key(|(_, d, _)| OrderedFloat(*d))
        {
            Some(x) => x,
            None => {
                return MapProject {
                    pos,
                    kind: ProjectKind::Ground,
                }
            }
        };

        if self.intersections[min_road.src].polygon.contains(pos) {
            return MapProject {
                pos: self.intersections[min_road.src].pos,
                kind: ProjectKind::Inter(min_road.src),
            };
        }
        if self.intersections[min_road.dst].polygon.contains(pos) {
            return MapProject {
                pos: self.intersections[min_road.dst].pos,
                kind: ProjectKind::Inter(min_road.dst),
            };
        }

        if d < THRESHOLD * THRESHOLD {
            MapProject {
                pos: projected,
                kind: ProjectKind::Road(min_road.id),
            }
        } else {
            MapProject {
                pos,
                kind: ProjectKind::Ground,
            }
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

    pub fn get_random_lane<R: Rng>(&self, kind: LaneKind, r: &mut R) -> Option<&Lane> {
        let (_, road) = self.roads.iter().choose(r)?;
        let lanes = road
            .lanes_iter()
            .filter(|x| self.lanes[**x].kind == kind)
            .collect::<Vec<&LaneID>>();

        lanes.iter().choose(r).map(|x| &self.lanes[**x])
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

    pub fn is_neigh(&self, src: IntersectionID, dst: IntersectionID) -> bool {
        self.find_road(src, dst).is_some()
    }
}
