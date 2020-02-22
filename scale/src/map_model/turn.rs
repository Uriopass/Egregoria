use crate::geometry::splines::Spline;
use crate::map_model::{IntersectionID, LaneID, Lanes, NavMesh, NavNode, NavNodeID};
use cgmath::{Array, InnerSpace};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnID(pub usize);

#[derive(Serialize, Deserialize)]
pub struct Turn {
    pub parent: IntersectionID,
    pub src: LaneID,
    pub dst: LaneID,

    easing_nodes: Vec<NavNodeID>,
    generated: bool,
}

impl Turn {
    pub fn new(parent: IntersectionID, src: LaneID, dst: LaneID) -> Self {
        Self {
            parent,
            src,
            dst,
            easing_nodes: vec![],
            generated: false,
        }
    }

    pub fn gen_navmesh(&mut self, lanes: &Lanes, navmesh: &mut NavMesh) {
        if self.is_generated() {
            panic!("Turn already generated !");
        }

        const N_SPLINE: usize = 3;

        let src_lane = &lanes[self.src];
        let dst_lane = &lanes[self.dst];

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

        self.generated = true;

        self.reposition_nodes(lanes, navmesh);
    }

    pub fn is_generated(&self) -> bool {
        self.generated
    }

    pub fn clean(&mut self, navmesh: &mut NavMesh) {
        for x in self.easing_nodes.drain(0..) {
            navmesh.remove_node(x);
        }
        self.generated = false;
    }

    pub fn reposition_nodes(&mut self, lanes: &Lanes, navmesh: &mut NavMesh) {
        let src_lane = &lanes[self.src];
        let dst_lane = &lanes[self.dst];

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

            let pos = spline.get(c);
            assert!(pos.is_finite());
            navmesh.get_mut(*node).unwrap().pos = spline.get(c);
        }
    }
}
