use crate::{
    House, HouseID, Intersection, IntersectionID, Lane, LaneID, LaneKind, LanePattern,
    ParkingSpotID, ParkingSpots, Road, RoadID, RoadSegmentKind, SpatialMap,
};
use geom::splines::Spline;
use geom::Vec2;
use ordered_float::OrderedFloat;
use rand::prelude::IteratorRandom;
use rand::Rng;
use slotmap::DenseSlotMap;

pub type Roads = DenseSlotMap<RoadID, Road>;
pub type Lanes = DenseSlotMap<LaneID, Lane>;
pub type Intersections = DenseSlotMap<IntersectionID, Intersection>;
pub type Houses = DenseSlotMap<HouseID, House>;
#[derive(Debug, Clone, Copy)]
pub enum ProjectKind {
    Inter(IntersectionID),
    Road(RoadID),
    House(HouseID),
    Ground,
}

#[derive(Debug, Clone, Copy)]
pub struct MapProject {
    pub pos: Vec2,
    pub kind: ProjectKind,
}

pub struct Map {
    pub(crate) roads: Roads,
    pub(crate) lanes: Lanes,
    pub(crate) intersections: Intersections,
    pub(crate) houses: Houses,
    pub(crate) spatial_map: SpatialMap,
    pub parking: ParkingSpots,
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
            roads: Roads::default(),
            lanes: Lanes::default(),
            intersections: Intersections::default(),
            parking: ParkingSpots::default(),
            houses: Houses::default(),
            dirty: true,
            spatial_map: SpatialMap::default(),
        }
    }

    pub fn update_intersection(&mut self, id: IntersectionID, f: impl Fn(&mut Intersection) -> ()) {
        println!("update_intersection {:?}", id);
        let inter = unwrap_or!(self.intersections.get_mut(id), return);
        f(inter);

        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);
    }

    fn invalidate(&mut self, id: IntersectionID) {
        println!("invalidate {:?}", id);

        self.dirty = true;
        let inter = &mut self.intersections[id];
        inter.update_interface_radius(&mut self.roads);

        for x in inter.roads.clone() {
            let other_end = &mut self.intersections[self.roads[x].other_end(id)];
            other_end.update_interface_radius(&mut self.roads);

            let road = &mut self.roads[x];
            road.gen_pos(&self.intersections, &mut self.lanes, &mut self.parking);

            let other_end = &mut self.intersections[self.roads[x].other_end(id)];
            other_end.update_polygon(&self.roads);
        }

        let inter = &mut self.intersections[id];
        inter.update_traffic_control(&mut self.lanes, &self.roads);
        inter.update_turns(&self.lanes, &self.roads);
        inter.update_polygon(&self.roads);
    }

    pub fn add_intersection(&mut self, pos: Vec2) -> IntersectionID {
        println!("add_intersection {:?}", pos);
        self.dirty = true;
        Intersection::make(&mut self.intersections, pos)
    }

    pub fn remove_intersection(&mut self, src: IntersectionID) {
        println!("remove_intersection {:?}", src);

        self.dirty = true;
        for road in self.intersections[src].roads.clone() {
            self.remove_road(road);
        }

        self.intersections.remove(src);
    }

    pub fn remove_house(&mut self, h: HouseID) -> Option<House> {
        let h = self.houses.remove(h);
        self.dirty |= h.is_some();
        h
    }

    pub fn split_road(&mut self, id: RoadID, pos: Vec2) -> IntersectionID {
        println!("split_road {:?} {:?}", id, pos);

        let r = self
            .remove_road(id)
            .expect("Trying to split unexisting road");
        let id = self.add_intersection(pos);

        let pat = r.pattern();
        match r.segment {
            RoadSegmentKind::Straight => {
                self.connect(r.src, id, &pat, RoadSegmentKind::Straight);
                self.connect(id, r.dst, &pat, RoadSegmentKind::Straight);
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
                    &pat,
                    RoadSegmentKind::Curved((s_from.from_derivative, s_from.to_derivative)),
                );
                self.connect(
                    id,
                    r.dst,
                    &pat,
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
        pattern: &LanePattern,
    ) -> RoadID {
        self.connect(src, dst, pattern, RoadSegmentKind::Straight)
    }

    pub fn connect(
        &mut self,
        src: IntersectionID,
        dst: IntersectionID,
        pattern: &LanePattern,
        segment: RoadSegmentKind,
    ) -> RoadID {
        println!("connect {:?} {:?} {:?} {:?}", src, dst, pattern, segment);

        self.dirty = true;
        let road_id = Road::make(src, dst, segment, pattern, self);

        let inters = &mut self.intersections;

        inters[src].add_road(road_id, &self.roads);
        inters[dst].add_road(road_id, &self.roads);

        self.invalidate(src);
        self.invalidate(dst);

        self.make_houses(road_id);

        road_id
    }

    fn make_houses(&mut self, road: RoadID) {
        let r = &self.roads[road];

        let l = r.generated_points.length();
        let w = r.width * 0.5;

        for (pos, dir) in r
            .generated_points
            .points_dirs_along((0..(l / 30.0) as usize).map(|i| i as f32 * 30.0 + 15.0))
            .collect::<Vec<_>>()
        {
            let p = dir.perpendicular();
            House::try_make(self, pos + p * (w + 3.0), p);
            House::try_make(self, pos - p * (w + 3.0), -p);
        }
    }

    pub fn remove_road(&mut self, road_id: RoadID) -> Option<Road> {
        println!("remove_road {:?}", road_id);

        self.dirty = true;
        let road = self.roads.remove(road_id)?;

        self.spatial_map.remove_road(&road);

        for (id, _) in road.lanes_iter() {
            self.lanes.remove(id);
            self.parking.remove_spots(id);
        }

        self.intersections[road.src].remove_road(road_id);
        self.intersections[road.dst].remove_road(road_id);

        self.invalidate(road.src);
        self.invalidate(road.dst);
        Some(road)
    }

    pub fn clear(&mut self) {
        println!("clear");

        self.dirty = true;
        self.intersections.clear();
        self.lanes.clear();
        self.roads.clear();
        self.parking.clear();
    }

    pub fn project(&self, pos: Vec2) -> MapProject {
        let mk_proj = move |kind| MapProject { pos, kind };

        for obj in self.spatial_map.query_point(pos) {
            match obj {
                ProjectKind::Road(id) => {
                    let road = self.roads
                        .get(id)
                        .expect("Road does not exist anymore, you seem to have forgotten to remove it from the spatial map.");

                    if self.intersections[road.src].polygon.contains(pos) {
                        return MapProject {
                            pos: self.intersections[road.src].pos,
                            kind: ProjectKind::Inter(road.src),
                        };
                    }
                    if self.intersections[road.dst].polygon.contains(pos) {
                        return MapProject {
                            pos: self.intersections[road.dst].pos,
                            kind: ProjectKind::Inter(road.dst),
                        };
                    }

                    let projected = road.generated_points.project(pos);
                    if projected.distance(pos) < road.width {
                        return MapProject {
                            pos: projected,
                            kind: ProjectKind::Road(id),
                        };
                    }
                },
                ProjectKind::House(id) => {
                    if self.houses
                        .get(id)
                        .expect("House does not exist anymore, you seem to have forgotten to remove it from the spatial map.")
                        .exterior
                        .contains(pos) {
                        return mk_proj(ProjectKind::House(id));
                    }
                },
                _ => {},
            }
        }

        mk_proj(ProjectKind::Ground)
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
    pub fn houses(&self) -> &Houses {
        &self.houses
    }

    pub fn get_random_lane<R: Rng>(&self, filter: LaneKind, r: &mut R) -> Option<&Lane> {
        self.roads
            .iter()
            .choose(r)?
            .1
            .lanes_iter()
            .filter(|&(_, kind)| kind == filter)
            .map(|(id, _)| id)
            .choose(r)
            .map(|x| &self.lanes[x])
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

    pub fn parking_to_drive(&self, spot: ParkingSpotID) -> Option<LaneID> {
        let spot = self.parking.get(spot)?;
        let park_lane = self
            .lanes
            .get(spot.parent)
            .expect("Parking spot has no parent >:(");
        let road = self
            .roads
            .get(park_lane.parent)
            .expect("Lane has no parent >:(");
        road.outgoing_lanes_from(park_lane.src)
            .iter()
            .rfind(|&&(_, kind)| kind == LaneKind::Driving)
            .map(|&(id, _)| id)
    }
}
