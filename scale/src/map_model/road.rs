use crate::map_model::{Intersection, IntersectionID, Lane, LaneID, NavMesh};
use cgmath::InnerSpace;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use slab::Slab;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadID(pub usize);

#[derive(Serialize, Deserialize)]
pub struct Road {
    pub id: RoadID,
    pub src: IntersectionID,
    pub dst: IntersectionID,

    pub interpolation_points: Vec<Vector2<f32>>,

    pub lanes_forward: Vec<LaneID>,
    pub lanes_backward: Vec<LaneID>,
}

impl Road {
    pub fn make<'a>(
        store: &'a mut Slab<Road>,
        intersections: &Slab<Intersection>,
        src: IntersectionID,
        dst: IntersectionID,
    ) -> &'a mut Self {
        let pos_src = intersections[src.0].pos;
        let pos_dst = intersections[dst.0].pos;

        let entry = store.vacant_entry();
        let id = RoadID(entry.key());
        entry.insert(Self {
            id,
            src,
            dst,
            interpolation_points: vec![pos_src, pos_dst],
            lanes_forward: vec![],
            lanes_backward: vec![],
        })
    }

    pub fn gen_navmesh(
        &mut self,
        intersections: &Slab<Intersection>,
        lanes: &mut Slab<Lane>,
        navmesh: &mut NavMesh,
    ) {
        for lane in &self.lanes_forward {
            let lane = &mut lanes[lane.0];
            if lane.src_node.is_some() {
                continue;
            }
            lane.src_node = Some(intersections[lane.src_i.0].out_nodes[&lane.id]);
            lane.dst_node = Some(intersections[lane.dst_i.0].in_nodes[&lane.id]);

            navmesh.add_neigh(lane.src_node.unwrap(), lane.dst_node.unwrap(), 1.0);
        }

        for lane in &self.lanes_backward {
            let lane = &mut lanes[lane.0];
            if lane.src_node.is_some() {
                continue;
            }
            lane.src_node = Some(intersections[lane.src_i.0].in_nodes[&lane.id]);
            lane.dst_node = Some(intersections[lane.dst_i.0].out_nodes[&lane.id]);

            navmesh.add_neigh(lane.dst_node.unwrap(), lane.src_node.unwrap(), 1.0);
        }

        self.interpolation_points[0] = intersections[self.src.0].pos;
        let l = self.interpolation_points.len();
        self.interpolation_points[l - 1] = intersections[self.dst.0].pos;
    }

    pub fn dir_from(&self, i: &Intersection) -> Vector2<f32> {
        if i.id == self.src {
            (self.interpolation_points[1] - i.pos).normalize()
        } else if i.id == self.dst {
            (self.interpolation_points[self.interpolation_points.len() - 2] - i.pos).normalize()
        } else {
            panic!("Asking dir from from an intersection not conected to the road");
        }
    }

    pub fn other_end(&self, my_end: IntersectionID) -> IntersectionID {
        if self.src == my_end {
            return self.dst;
        } else if self.dst == my_end {
            return self.src;
        }
        panic!(
            "Asking other end of {:?} which isn't connected to {:?}",
            self.id, my_end
        );
    }

    pub fn idx_unchecked(&self, lane: LaneID) -> usize {
        if let Some((x, _)) = self
            .lanes_backward
            .iter()
            .enumerate()
            .find(|(_, x)| **x == lane)
        {
            return x + 1;
        }
        if let Some((x, _)) = self
            .lanes_forward
            .iter()
            .enumerate()
            .find(|(_, x)| **x == lane)
        {
            return x + 1;
        }
        0
    }
}
