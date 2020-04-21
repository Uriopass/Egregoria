use crate::geometry::Vec2;
use crate::map_model::{
    Intersection, IntersectionID, Lane, LaneID, LaneKind, LanePattern, LightPolicy, Road, RoadID,
    TurnPolicy,
};
use crate::utils::rand_det;
use serde::{Deserialize, Serialize};
use slotmap::DenseSlotMap;

pub type Roads = DenseSlotMap<RoadID, Road>;
pub type Lanes = DenseSlotMap<LaneID, Lane>;
pub type Intersections = DenseSlotMap<IntersectionID, Intersection>;

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

    pub fn set_intersection_radius(&mut self, id: IntersectionID, radius: f32) {
        if (self.intersections[id].interface_radius - radius).abs() < 0.001 {
            return;
        }
        self.intersections[id].interface_radius = radius;
        for x in &self.intersections[id].roads {
            self.roads[*x].gen_pos(&self.intersections, &mut self.lanes);
        }
        self.intersections[id].gen_turns(&self.lanes, &self.roads);
    }

    pub fn set_intersection_turn_policy(&mut self, id: IntersectionID, policy: TurnPolicy) {
        if self.intersections[id].turn_policy == policy {
            return;
        }

        self.intersections[id].turn_policy = policy;
        self.intersections[id].gen_turns(&self.lanes, &self.roads);
    }

    pub fn set_intersection_light_policy(&mut self, id: IntersectionID, policy: LightPolicy) {
        if self.intersections[id].light_policy == policy {
            return;
        }

        self.intersections[id].light_policy = policy;
        self.intersections[id].update_traffic_control(&mut self.lanes, &self.roads);
    }

    pub fn add_intersection(&mut self, pos: Vec2) -> IntersectionID {
        Intersection::make(&mut self.intersections, pos)
    }

    pub fn move_intersection(&mut self, id: IntersectionID, pos: Vec2) {
        self.intersections[id].pos = pos;

        for x in self.intersections[id].roads.clone() {
            self.roads[x].gen_pos(&self.intersections, &mut self.lanes);

            let other_end = &mut self.intersections[self.roads[x].other_end(id)];
            other_end.gen_turns(&self.lanes, &self.roads);
            other_end.update_traffic_control(&mut self.lanes, &self.roads);
        }

        self.intersections[id].gen_turns(&self.lanes, &self.roads);
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
        pattern: &LanePattern,
    ) -> RoadID {
        let road_id = Road::make(
            &mut self.roads,
            &self.intersections,
            src,
            dst,
            &mut self.lanes,
            &pattern,
        );

        self.intersections[src].add_road(road_id, &mut self.lanes, &self.roads);
        self.intersections[dst].add_road(road_id, &mut self.lanes, &self.roads);

        road_id
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

    pub(crate) fn remove_road(&mut self, road_id: RoadID) -> Road {
        let road = self.roads.remove(road_id).unwrap();
        for lane_id in road.lanes_iter() {
            self.lanes.remove(*lane_id).unwrap();
        }

        self.intersections[road.src].remove_road(road_id, &mut self.lanes, &self.roads);
        self.intersections[road.dst].remove_road(road_id, &mut self.lanes, &self.roads);

        road
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
