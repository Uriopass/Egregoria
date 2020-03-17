use crate::geometry::splines::Spline;
use crate::map_model::{IntersectionID, LaneID, Lanes};
use cgmath::{Array, InnerSpace, Vector2};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, PartialOrd, Ord, Deserialize, PartialEq, Eq)]
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
    pub points: Vec<Vector2<f32>>,
}

impl Turn {
    pub fn new(id: TurnID) -> Self {
        Self { id, points: vec![] }
    }

    pub(crate) fn make_points(&mut self, lanes: &Lanes) {
        const N_SPLINE: usize = 6;

        let src_lane = &lanes[self.id.src];
        let dst_lane = &lanes[self.id.dst];

        let pos_src = src_lane.get_inter_node_pos(self.id.parent);
        let pos_dst = dst_lane.get_inter_node_pos(self.id.parent);

        let dist = (pos_dst - pos_src).magnitude() / 2.0;

        let derivative_src = src_lane.get_orientation_vec() * dist;
        let derivative_dst = dst_lane.get_orientation_vec() * dist;

        let spline = Spline {
            from: pos_src,
            to: pos_dst,
            from_derivative: derivative_src,
            to_derivative: derivative_dst,
        };

        self.points.clear();
        for i in 0..N_SPLINE {
            let c = (i + 1) as f32 / (N_SPLINE + 1) as f32;

            let pos = spline.get(c);
            debug_assert!(pos.is_finite());
            self.points.push(pos);
        }
    }
}
