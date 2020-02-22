use crate::map_model::{
    Intersection, IntersectionID, Lane, LaneDirection, LaneID, LaneType, NavMesh, Road, RoadID,
};
use cgmath::Vector2;
use slotmap::DenseSlotMap;

pub type Roads = DenseSlotMap<RoadID, Road>;
pub type Lanes = DenseSlotMap<LaneID, Lane>;
pub type Intersections = DenseSlotMap<IntersectionID, Intersection>;

pub struct Map {
    pub roads: Roads,
    pub lanes: Lanes,
    pub intersections: Intersections,
    pub navmesh: NavMesh,
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

    pub fn set_intersection_radius(&mut self, id: IntersectionID, radius: f32) {
        self.intersections[id].interface_radius = radius;
        for x in &self.intersections[id].roads {
            self.roads[*x].gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);
        }
        self.intersections[id].gen_turns(&self.lanes, &mut self.navmesh);
    }

    pub fn add_intersection(&mut self, pos: Vector2<f32>) -> IntersectionID {
        Intersection::make(&mut self.intersections, pos)
    }

    pub fn move_intersection(&mut self, id: IntersectionID, pos: Vector2<f32>) {
        self.intersections[id].pos = pos;

        let inter = &self.intersections[id];
        for x in &inter.roads {
            self.roads[*x].gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);
        }

        self.intersections[id].gen_turns(&self.lanes, &mut self.navmesh);
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
        n_lanes: i32,
        one_way: bool,
    ) -> RoadID {
        let road_id = Road::make(&mut self.roads, &self.intersections, src, dst);

        let road = &mut self.roads[road_id];

        for _ in 0..n_lanes {
            road.add_lane(&mut self.lanes, LaneType::Driving, LaneDirection::Forward);
            if !one_way {
                road.add_lane(&mut self.lanes, LaneType::Driving, LaneDirection::Backward);
            }
        }

        road.gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);

        self.intersections[src].add_road(road);
        self.intersections[dst].add_road(road);

        self.intersections[src].gen_turns(&self.lanes, &mut self.navmesh);
        self.intersections[dst].gen_turns(&self.lanes, &mut self.navmesh);

        let id = road.id;
        self.intersections[src].update_traffic_lights(&self.roads, &self.lanes, &mut self.navmesh);
        self.intersections[dst].update_traffic_lights(&self.roads, &self.lanes, &mut self.navmesh);
        id
    }

    pub fn disconnect(&mut self, src: IntersectionID, dst: IntersectionID) {
        let r = self.find_road(src, dst);
        let road_id = match r {
            None => return,
            Some(x) => x,
        };

        self.remove_road(road_id);
    }

    fn remove_road(&mut self, road_id: RoadID) {
        let road = self.roads.remove(road_id).unwrap();
        for lane_id in road
            .lanes_forward
            .into_iter()
            .chain(road.lanes_backward.into_iter())
        {
            let mut lane = self.lanes.remove(lane_id).unwrap();
            lane.clean(&mut self.navmesh);
        }

        self.intersections[road.src].clean(&self.lanes, &self.roads, &mut self.navmesh);
        self.intersections[road.dst].clean(&self.lanes, &self.roads, &mut self.navmesh);
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

    pub fn is_neigh(&self, src: IntersectionID, dst: IntersectionID) -> bool {
        self.find_road(src, dst).is_some()
    }

    /*
    pub fn from_file(filename: &'static str) -> Option<NavMesh> {
        let f = File::open(filename.to_string() + ".bc").ok()?;
        bincode::deserialize_from(f).ok()
    }

    pub fn save(&self, filename: &'static str) {
        let file =
            File::create(filename.to_string() + ".bc").expect("Could not open file for saving map");
        bincode::serialize_into(file, self).unwrap();
    }
    */
}
