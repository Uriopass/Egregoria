use crate::map_model::{
    Intersection, IntersectionID, Lane, LaneID, LanePattern, NavMesh, Road, RoadID, TurnPolicy,
};
use cgmath::Vector2;
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
    navmesh: NavMesh,
}

impl Map {
    pub fn empty() -> Self {
        Self {
            roads: Roads::with_key(),
            lanes: Lanes::with_key(),
            intersections: Intersections::with_key(),
            navmesh: NavMesh::empty(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.roads.is_empty()
            && self.lanes.is_empty()
            && self.intersections.is_empty()
            && self.navmesh.is_empty()
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
    pub fn navmesh(&self) -> &NavMesh {
        &self.navmesh
    }

    pub fn set_intersection_radius(&mut self, id: IntersectionID, radius: f32) {
        if (self.intersections[id].interface_radius - radius).abs() < 0.001 {
            return;
        }
        self.intersections[id].interface_radius = radius;
        for x in &self.intersections[id].roads {
            self.roads[*x].gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);
        }
        self.intersections[id].gen_turns(&self.lanes, &self.roads, &mut self.navmesh);
    }

    pub fn set_intersection_turn_policy(&mut self, id: IntersectionID, policy: TurnPolicy) {
        if self.intersections[id].policy == policy {
            return;
        }

        self.intersections[id].policy = policy;
        self.intersections[id].gen_turns(&self.lanes, &self.roads, &mut self.navmesh);
    }

    pub fn add_intersection(&mut self, pos: Vector2<f32>) -> IntersectionID {
        Intersection::make(&mut self.intersections, pos)
    }

    pub fn move_intersection(&mut self, id: IntersectionID, pos: Vector2<f32>) {
        self.intersections[id].pos = pos;

        for x in self.intersections[id].roads.clone() {
            self.roads[x].gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);
            self.intersections[self.roads[x].other_end(id)].gen_turns(
                &self.lanes,
                &self.roads,
                &mut self.navmesh,
            );
        }

        self.intersections[id].gen_turns(&self.lanes, &self.roads, &mut self.navmesh);
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

        let road = &mut self.roads[road_id];

        road.gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);

        self.intersections[src].add_road(road);
        self.intersections[dst].add_road(road);

        let id = road.id;

        self.intersections[src].gen_turns(&self.lanes, &self.roads, &mut self.navmesh);
        self.intersections[dst].gen_turns(&self.lanes, &self.roads, &mut self.navmesh);

        self.intersections[src].update_traffic_control(&self.roads, &self.lanes, &mut self.navmesh);
        self.intersections[dst].update_traffic_control(&self.roads, &self.lanes, &mut self.navmesh);
        id
    }

    pub fn disconnect(&mut self, src: IntersectionID, dst: IntersectionID) -> Option<Road> {
        let r = self.find_road(src, dst);
        let road_id = r?;
        let r = self.remove_road(road_id);

        self.intersections[src].update_traffic_control(&self.roads, &self.lanes, &mut self.navmesh);
        self.intersections[dst].update_traffic_control(&self.roads, &self.lanes, &mut self.navmesh);

        Some(r)
    }

    fn remove_road(&mut self, road_id: RoadID) -> Road {
        let road = self.roads.remove(road_id).unwrap();
        for lane_id in road.lanes_forward.iter().chain(road.lanes_backward.iter()) {
            let mut lane = self.lanes.remove(*lane_id).unwrap();
            lane.clean(&mut self.navmesh);
        }

        self.intersections[road.src].clean(&self.lanes, &self.roads, &mut self.navmesh);
        self.intersections[road.dst].clean(&self.lanes, &self.roads, &mut self.navmesh);

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

    pub fn closest_lane(&self, p: Vector2<f32>) -> Option<LaneID> {
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
