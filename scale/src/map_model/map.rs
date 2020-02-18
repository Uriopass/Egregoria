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
        let inter = &mut self.intersections[id];
        inter.interface_radius = radius;
        inter.gen_interface_navmesh(&mut self.lanes, &mut self.roads, &mut self.navmesh);
        inter.gen_turns(&mut self.lanes, &mut self.navmesh);
    }

    pub fn add_intersection(&mut self, pos: Vector2<f32>) -> IntersectionID {
        Intersection::make(&mut self.intersections, pos)
    }

    pub fn remove_intersection(&mut self, id: IntersectionID) {
        let inter = &mut self.intersections[id];
        for turn in &mut inter.turns {
            turn.clean(&mut self.navmesh);
        }

        todo!()
    }

    pub fn move_intersection(&mut self, id: IntersectionID, pos: Vector2<f32>) {
        self.intersections[id].pos = pos;

        let inter = &self.intersections[id];

        let mut to_update = vec![inter.id];

        for x in &inter.roads {
            let other = self.roads[*x].other_end(inter.id);
            to_update.push(other);

            self.roads[*x].gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);
        }

        for id in to_update {
            let inter = &mut self.intersections[id];
            inter.gen_interface_navmesh(&mut self.lanes, &self.roads, &mut self.navmesh);
            inter.gen_turns(&self.lanes, &mut self.navmesh);
        }
    }

    pub fn connect(&mut self, a: IntersectionID, b: IntersectionID, n_lanes: i32) -> RoadID {
        let road_id = Road::make(&mut self.roads, &self.intersections, a, b);

        let road = &mut self.roads[road_id];

        for _ in 0..n_lanes {
            road.add_lane(&mut self.lanes, LaneType::Driving, LaneDirection::Forward);
            road.add_lane(&mut self.lanes, LaneType::Driving, LaneDirection::Backward);
        }

        self.intersections[a].add_road(road);
        self.intersections[b].add_road(road);

        let road = road.id;

        self.intersections[a].gen_interface_navmesh(
            &mut self.lanes,
            &self.roads,
            &mut self.navmesh,
        );

        self.intersections[b].gen_interface_navmesh(
            &mut self.lanes,
            &self.roads,
            &mut self.navmesh,
        );

        self.roads[road].gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);

        self.intersections[a].gen_turns(&self.lanes, &mut self.navmesh);
        self.intersections[b].gen_turns(&self.lanes, &mut self.navmesh);
        road
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
