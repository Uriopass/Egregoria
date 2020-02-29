use crate::geometry::splines::Spline;
use crate::map_model::{
    Intersection, IntersectionID, LaneID, Lanes, NavMesh, NavNode, NavNodeID, Roads,
};
use cgmath::{vec2, Array, InnerSpace};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnID {
    pub parent: IntersectionID,
    pub src: LaneID,
    pub dst: LaneID,
}

impl TurnID {
    pub fn new(parent: IntersectionID, src: LaneID, dst: LaneID) -> Self {
        Self { parent, src, dst }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Turn {
    pub id: TurnID,
    easing_nodes: Vec<NavNodeID>,
    generated: bool,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Inspect)]
pub struct TurnPolicy {
    back_turns: bool,
    left_turns: bool,
}

impl Default for TurnPolicy {
    fn default() -> Self {
        Self {
            back_turns: false,
            left_turns: true,
        }
    }
}

impl Turn {
    pub fn new(id: TurnID) -> Self {
        Self {
            id,
            easing_nodes: vec![],
            generated: false,
        }
    }

    pub fn gen_navmesh(&mut self, lanes: &Lanes, navmesh: &mut NavMesh) {
        if self.is_generated() {
            panic!("Turn already generated !");
        }

        const N_SPLINE: usize = 3;

        let src_lane = &lanes[self.id.src];
        let dst_lane = &lanes[self.id.dst];

        let node_src = src_lane.get_inter_node(self.id.parent);
        let node_dst = dst_lane.get_inter_node(self.id.parent);

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
        let src_lane = &lanes[self.id.src];
        let dst_lane = &lanes[self.id.dst];

        let node_src = src_lane.get_inter_node(self.id.parent);
        let node_dst = dst_lane.get_inter_node(self.id.parent);

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

impl TurnPolicy {
    fn zip(inter_id: IntersectionID, incoming: &[LaneID], outgoing: &[LaneID]) -> Vec<TurnID> {
        incoming
            .iter()
            .zip(outgoing)
            .map(|(lane_src, lane_dst)| TurnID::new(inter_id, *lane_src, *lane_dst))
            .collect()
    }

    fn all(inter_id: IntersectionID, incoming: &[LaneID], outgoing: &[LaneID]) -> Vec<TurnID> {
        incoming
            .iter()
            .map(|lane_src| {
                outgoing
                    .iter()
                    .map(move |lane_dst| TurnID::new(inter_id, *lane_src, *lane_dst))
            })
            .flatten()
            .collect()
    }

    fn zip_on_same_length(
        inter_id: IntersectionID,
        incoming: &[LaneID],
        outgoing: &[LaneID],
    ) -> Vec<TurnID> {
        if incoming.len() == outgoing.len() {
            Self::zip(inter_id, incoming, outgoing)
        } else {
            Self::all(inter_id, incoming, outgoing)
        }
    }

    pub fn generate_turns(
        self,
        inter: &Intersection,
        lanes: &Lanes,
        roads: &Roads,
        mesh: &NavMesh,
    ) -> Vec<TurnID> {
        if inter.roads.len() == 1 {
            return Self::zip_on_same_length(
                inter.id,
                &inter.incoming_lanes,
                &inter.outgoing_lanes,
            );
        }

        let mut turns = vec![];

        if inter.roads.len() == 2 {
            let road1 = &roads[inter.roads[0]];
            let road2 = &roads[inter.roads[1]];

            let incoming_road1 = road1.incoming_lanes_from(inter.id);
            let incoming_road2 = road2.incoming_lanes_from(inter.id);

            let outgoing_road1 = road1.outgoing_lanes_from(inter.id);
            let outgoing_road2 = road2.outgoing_lanes_from(inter.id);

            turns.extend(Self::zip_on_same_length(
                inter.id,
                incoming_road1,
                outgoing_road2,
            ));

            turns.extend(Self::zip_on_same_length(
                inter.id,
                incoming_road2,
                outgoing_road1,
            ));

            return turns;
        }

        for incoming in &inter.incoming_lanes {
            for outgoing in &inter.outgoing_lanes {
                if lanes[*incoming].parent == lanes[*outgoing].parent && !self.back_turns {
                    continue;
                }
                let incoming_dir = lanes[*incoming].get_orientation_vec(mesh);
                let outgoing_dir = lanes[*outgoing].get_orientation_vec(mesh);

                let incoming_right = vec2(incoming_dir.y, -incoming_dir.x);
                let id = TurnID::new(inter.id, *incoming, *outgoing);

                if self.left_turns || incoming_right.dot(outgoing_dir) >= -0.3 {
                    turns.push(id);
                }
            }
        }

        turns
    }
}
