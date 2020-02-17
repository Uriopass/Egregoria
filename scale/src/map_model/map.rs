use crate::map_model::{Intersection, IntersectionID, Lane, LaneType, NavMesh, Road, RoadID};
use cgmath::Vector2;
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

    pub fn add_intersection(&mut self, pos: Vector2<f32>) -> IntersectionID {
        Intersection::make(&mut self.intersections, pos).id
    }

    pub fn move_intersection(&mut self, id: IntersectionID, pos: Vector2<f32>) {
        self.intersections[id.0].pos = pos;

        let inter = &self.intersections[id.0];

        let mut to_update = vec![inter.id];

        for x in &inter.roads {
            let other = self.roads[x.0].other_end(inter.id);
            to_update.push(other);

            self.roads[x.0].gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);
        }

        for id in to_update {
            let inter = &mut self.intersections[id.0];
            inter.gen_interface_navmesh(&mut self.lanes, &self.roads, &mut self.navmesh);
            inter.gen_turns(&self.lanes, &mut self.navmesh);
        }
    }

    pub fn connect(&mut self, a: IntersectionID, b: IntersectionID) -> RoadID {
        let road = Road::make(&mut self.roads, &self.intersections, a, b);

        Lane::make_forward(&mut self.lanes, road, LaneType::Driving);
        Lane::make_forward(&mut self.lanes, road, LaneType::Driving);

        Lane::make_backward(&mut self.lanes, road, LaneType::Driving);
        Lane::make_backward(&mut self.lanes, road, LaneType::Driving);

        self.intersections[a.0].add_road(road);
        self.intersections[b.0].add_road(road);

        let road = road.id;

        self.intersections[a.0].gen_interface_navmesh(
            &mut self.lanes,
            &self.roads,
            &mut self.navmesh,
        );

        self.intersections[b.0].gen_interface_navmesh(
            &mut self.lanes,
            &self.roads,
            &mut self.navmesh,
        );

        self.roads[road.0].gen_navmesh(&self.intersections, &mut self.lanes, &mut self.navmesh);

        self.intersections[a.0].gen_turns(&self.lanes, &mut self.navmesh);
        self.intersections[b.0].gen_turns(&self.lanes, &mut self.navmesh);
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
