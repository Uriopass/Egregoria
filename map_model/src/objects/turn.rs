use crate::{IntersectionID, LaneID, Lanes};
use geom::PolyLine3;
use geom::{Spline, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TurnKind {
    Crosswalk,
    WalkingCorner,
    Driving,
    Rail,
}

impl TurnKind {
    pub fn is_crosswalk(self) -> bool {
        matches!(self, TurnKind::Crosswalk)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn {
    pub id: TurnID,
    pub points: PolyLine3,
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
            points: PolyLine3::new(vec![Vec3::ZERO; N_SPLINE + 2]),
            kind,
        }
    }

    pub fn make_points(&mut self, lanes: &Lanes) {
        let src_lane = unwrap_ret!(lanes.get(self.id.src));
        let dst_lane = unwrap_ret!(lanes.get(self.id.dst));

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
            from: pos_src.xy(),
            to: pos_dst.xy(),
            from_derivative: derivative_src,
            to_derivative: derivative_dst,
        };

        self.points.extend(
            spline
                .smart_points(0.3, 0.0, 1.0)
                .skip(1)
                .map(|x| x.z(pos_src.z)),
        );
    }
}
