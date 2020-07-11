use crate::{IntersectionID, LaneID, Lanes};
use geom::polyline::PolyLine;
use geom::splines::Spline;
use geom::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, PartialOrd, Ord, Deserialize, PartialEq, Eq, Hash)]
pub struct TurnID {
    pub parent: IntersectionID,
    pub src: LaneID,
    pub dst: LaneID,
    pub bidirectional: bool,
}

impl TurnID {
    pub fn new(parent: IntersectionID, src: LaneID, dst: LaneID, bidirectional: bool) -> Self {
        Self {
            parent,
            src,
            dst,
            bidirectional,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialOrd, Ord, PartialEq, Serialize, Deserialize)]
pub enum TurnKind {
    Crosswalk,
    WalkingCorner,
    Driving,
}

impl TurnKind {
    pub fn is_crosswalk(self) -> bool {
        matches!(self, TurnKind::Crosswalk)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Turn {
    pub id: TurnID,
    pub points: PolyLine,
    pub kind: TurnKind,
}

const TURN_ANG_ADD: f32 = 0.29;
const TURN_ANG_MUL: f32 = 0.36;
const TURN_MUL: f32 = 0.46;
const N_SPLINE: usize = 6;

impl Turn {
    pub fn new(id: TurnID, kind: TurnKind) -> Self {
        Self {
            id,
            points: PolyLine::new(vec![Vec2::ZERO; N_SPLINE + 2]),
            kind,
        }
    }

    pub fn make_points(&mut self, lanes: &Lanes) {
        let src_lane = &lanes[self.id.src];
        let dst_lane = &lanes[self.id.dst];

        let pos_src = src_lane.get_inter_node_pos(self.id.parent);
        let pos_dst = dst_lane.get_inter_node_pos(self.id.parent);

        self.points.clear_push(pos_src);

        if self.kind.is_crosswalk() {
            self.points.push(pos_dst);
            return;
        }

        let src_dir = -src_lane.orientation_from(self.id.parent);
        let dst_dir = dst_lane.orientation_from(self.id.parent);

        let ang = src_dir.angle(dst_dir);

        let dist =
            (pos_dst - pos_src).magnitude() * (TURN_ANG_ADD + ang.abs() * TURN_ANG_MUL) * TURN_MUL;

        let derivative_src = src_dir * dist;
        let derivative_dst = dst_dir * dist;

        let spline = Spline {
            from: pos_src,
            to: pos_dst,
            from_derivative: derivative_src,
            to_derivative: derivative_dst,
        };

        self.points
            .extend(spline.smart_points(0.3, 0.0, 1.0).skip(1));
    }
}
