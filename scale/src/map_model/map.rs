use crate::map_model::{Intersection, IntersectionID, Lane, LaneType, NavMesh, Road};
use slab::Slab;

pub struct Map {
    pub roads: Slab<Road>,
    pub lanes: Slab<Lane>,
    pub intersections: Slab<Intersection>,
    pub navmesh: NavMesh,
}

impl Map {
    pub fn empty() -> Self {
        Self {
            roads: Slab::new(),
            lanes: Slab::new(),
            intersections: Slab::new(),
            navmesh: NavMesh::empty(),
        }
    }

    pub fn connect(&mut self, a: IntersectionID, b: IntersectionID) {
        let road = Road::make(&mut self.roads, &self.intersections, a, b);

        Lane::make_forward(&mut self.lanes, road, LaneType::Driving);
        Lane::make_backward(&mut self.lanes, road, LaneType::Driving);

        self.intersections[a.0].add_road(road);
        self.intersections[b.0].add_road(road);
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
