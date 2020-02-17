use crate::geometry::splines::Spline;
use crate::map_model::{IntersectionID, Lane, LaneID, NavMesh, NavNode, NavNodeID};
use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};
use slab::Slab;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnID(pub usize);

#[derive(Serialize, Deserialize)]
pub struct Turn {
    pub parent: IntersectionID,
    pub src: LaneID,
    pub dst: LaneID,

    pub easing_nodes: Vec<NavNodeID>,
}

impl Turn {
    pub fn gen_navmesh(&mut self, lanes: &Slab<Lane>, navmesh: &mut NavMesh) {
        if !self.easing_nodes.is_empty() {
            panic!("Turn already generated !");
        }

        const N_SPLINE: usize = 5;

        let src_lane = &lanes[self.src.0];
        let dst_lane = &lanes[self.dst.0];

        let node_src = src_lane.get_inter_node(self.parent);
        let node_dst = dst_lane.get_inter_node(self.parent);

        for _ in 0..N_SPLINE {
            self.easing_nodes
                .push(navmesh.push(NavNode::new([0.0, 0.0].into())))
        }

        let mut v = vec![node_src];
        v.extend(&self.easing_nodes);
        v.push(node_dst);

        for x in v.windows(2) {
            navmesh.add_neigh(x[0], x[1], 1.0);
        }

        self.reposition_nodes(lanes, navmesh);
    }

    pub fn reposition_nodes(&mut self, lanes: &Slab<Lane>, navmesh: &mut NavMesh) {
        let src_lane = &lanes[self.src.0];
        let dst_lane = &lanes[self.dst.0];

        let node_src = src_lane.get_inter_node(self.parent);
        let node_dst = dst_lane.get_inter_node(self.parent);

        let pos_src = navmesh[node_src].pos;
        let pos_dst = navmesh[node_dst].pos;

        let dist = (pos_dst - pos_src).magnitude() / 2.0;

        let derivative_src = src_lane.get_orientation_vec(navmesh) * dist;
        let derivative_dst = dst_lane.get_orientation_vec(navmesh) * dist;

        let spline = Spline {
            from: pos_src,
            to: pos_dst,
            from_derivative: derivative_src,
            to_derivative: derivative_dst,
        };

        let len = self.easing_nodes.len();
        for (i, node) in self.easing_nodes.iter().enumerate() {
            let c = (i + 1) as f32 / (len + 1) as f32;

            navmesh.get_mut(*node).unwrap().pos = spline.get(c);
        }
    }
}
