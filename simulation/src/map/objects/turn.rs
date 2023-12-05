use crate::map::{Intersection, IntersectionID, LaneID, Lanes};
use geom::{Degrees, PolyLine3, Radians, Vec2};
use geom::{Spline, Vec3};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::f32::consts::TAU;

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

impl Borrow<TurnID> for Turn {
    fn borrow(&self) -> &TurnID {
        &self.id
    }
}

impl PartialOrd for Turn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Turn {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialEq for Turn {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for Turn {}

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

    pub fn make_points(&mut self, lanes: &Lanes, parent: &Intersection) {
        let Some(src_lane) = lanes.get(self.id.src) else {
            return;
        };
        let Some(dst_lane) = lanes.get(self.id.dst) else {
            return;
        };

        let pos_src = src_lane.get_inter_node_pos(self.id.parent);
        let pos_dst = dst_lane.get_inter_node_pos(self.id.parent);

        self.points.clear_push(pos_src);

        if self.kind.is_crosswalk() {
            self.points.push(pos_dst);
            return;
        }

        let src_dir = -src_lane.orientation_from(self.id.parent);
        let dst_dir = dst_lane.orientation_from(self.id.parent);

        if matches!(self.kind, TurnKind::Driving | TurnKind::WalkingCorner)
            && parent.is_roundabout()
        {
            if let Some(rp) = parent.turn_policy.roundabout {
                let center = parent.pos.xy();

                let center_dir_src = (pos_src.xy() - center).normalize();
                let center_dir_dst = (pos_dst.xy() - center).normalize();

                let ang = center_dir_dst.angle(center_dir_src).abs();
                if ang >= Radians::from(Degrees(21.0)).0 {
                    let a = center_dir_src.rotated_by_angle(Radians::from_deg(10.0));
                    let mut ang_a = a.angle_cossin();

                    let b = center_dir_dst.rotated_by_angle(Radians::from_deg(-10.0));
                    let ang_b = b.angle_cossin();

                    if ang_a > ang_b {
                        ang_a -= Radians::TAU;
                    }

                    let mut turn_radius = rp.radius * (1.0 - 0.5 * (ang_b.0 - ang_a.0) / TAU);

                    if matches!(self.kind, TurnKind::WalkingCorner) {
                        turn_radius = rp.radius + 3.0;
                    }

                    self.points.extend(
                        Self::gen_roundabout(
                            pos_src,
                            pos_dst,
                            src_dir,
                            dst_dir,
                            turn_radius,
                            center,
                        )
                        .skip(1)
                        .map(|x| x.z(pos_src.z)),
                    );
                    return;
                }
            }
        }

        let spline = Self::spline(pos_src.xy(), pos_dst.xy(), src_dir, dst_dir);

        self.points.extend(
            spline
                .smart_points(0.3, 0.0, 1.0)
                .skip(1)
                .map(|x| x.z(pos_src.z)),
        );
    }

    pub fn gen_roundabout(
        pos_src: Vec3,
        pos_dst: Vec3,
        src_dir: Vec2,
        dst_dir: Vec2,
        radius: f32,
        center: Vec2,
    ) -> impl Iterator<Item = Vec2> {
        let a = (pos_src.xy() - center)
            .normalize()
            .rotated_by_angle(Radians::from_deg(10.0));
        let mut ang_a = a.angle_cossin();

        let b = (pos_dst.xy() - center)
            .normalize()
            .rotated_by_angle(Radians::from_deg(-10.0));
        let mut ang_b = b.angle_cossin();

        if ang_a > ang_b {
            ang_a -= Radians::TAU;
        }

        let diff = (ang_b - ang_a).min(Radians::from_deg(80.0));

        ang_a += diff * 0.4;
        ang_b -= diff * 0.4;

        let a = ang_a.vec2();
        let b = ang_b.vec2();

        let sp1 = Self::spline(
            pos_src.xy(),
            center + a * radius,
            src_dir,
            -a.perpendicular(),
        );
        let sp2 = Self::circular_arc(center, ang_a, ang_b, radius);
        let sp3 = Self::spline(
            center + b * radius,
            pos_dst.xy(),
            -b.perpendicular(),
            dst_dir,
        );

        sp1.into_smart_points(0.3, 0.0, 1.0)
            .chain(sp2.skip(1))
            .chain(sp3.into_smart_points(0.3, 0.0, 1.0).skip(1))
    }

    /// Return points of a circular arc in counter-clockwise order from ang_a to ang_b, assuming ang_a < ang_b
    pub fn circular_arc(
        center: Vec2,
        ang_a: Radians,
        ang_b: Radians,
        radius: f32,
    ) -> impl Iterator<Item = Vec2> {
        const PRECISION: f32 = 1.0 / 0.1; // denominator is angular step in radians

        let ang = ang_b - ang_a;
        let n = (ang.0.abs() * PRECISION).ceil() as usize;
        let ang_step = Radians(ang.0 / n as f32);

        (0..=n).map(move |i| {
            let ang = ang_a + Radians(ang_step.0 * i as f32);
            center + ang.vec2() * radius
        })
    }

    /// Return points of a nice spline from `from` to `to` with derivatives `from_dir` and `to_dir`
    pub fn spline(from: Vec2, to: Vec2, from_dir: Vec2, to_dir: Vec2) -> Spline {
        let ang = from_dir.angle(to_dir);

        let dist = (to - from).mag() * (TURN_ANG_ADD + ang.abs() * TURN_ANG_MUL) * TURN_MUL;

        let derivative_src = from_dir * dist;
        let derivative_dst = to_dir * dist;

        Spline {
            from,
            to,
            from_derivative: derivative_src,
            to_derivative: derivative_dst,
        }
    }
}
