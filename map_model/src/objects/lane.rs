use crate::{IntersectionID, Lanes, Road, RoadID, TrafficControl, TraverseDirection};
use geom::PolyLine;
use geom::Vec2;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LaneID;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum LaneKind {
    Driving,
    Biking,
    Parking,
    Bus,
    Construction,
    Walking,
}

impl LaneKind {
    pub fn vehicles(self) -> bool {
        matches!(self, LaneKind::Driving | LaneKind::Biking | LaneKind::Bus)
    }

    pub fn needs_light(self) -> bool {
        matches!(self, LaneKind::Driving | LaneKind::Biking | LaneKind::Bus)
    }

    pub fn width(self) -> f32 {
        match self {
            LaneKind::Driving | LaneKind::Biking | LaneKind::Bus => 8.0,
            LaneKind::Parking => 4.0,
            LaneKind::Construction => 4.0,
            LaneKind::Walking => 4.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaneDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lane {
    pub id: LaneID,
    pub parent: RoadID,

    pub kind: LaneKind,

    pub control: TrafficControl,

    /// Src and dst implies direction
    pub src: IntersectionID,
    pub dst: IntersectionID,

    /// Always from src to dst
    pub points: PolyLine,
    pub width: f32,
    pub dist_from_bottom: f32,

    /// Length from start to end
    pub length: f32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LanePattern {
    pub lanes_forward: Vec<LaneKind>,
    pub lanes_backward: Vec<LaneKind>,
}

impl LanePattern {
    pub fn lanes(&self) -> impl Iterator<Item = (LaneKind, LaneDirection)> + '_ {
        self.lanes_forward
            .iter()
            .rev()
            .map(|&x| (x, LaneDirection::Forward))
            .chain(
                self.lanes_backward
                    .iter()
                    .map(|&x| (x, LaneDirection::Backward)),
            )
    }

    pub fn width(&self) -> f32 {
        self.lanes().map(|(kind, _)| kind.width()).sum()
    }
}

#[derive(Copy, Clone, Inspect)]
pub struct LanePatternBuilder {
    pub n_lanes: u32,
    pub sidewalks: bool,
    pub parking: bool,
    pub one_way: bool,
}

impl Default for LanePatternBuilder {
    fn default() -> Self {
        LanePatternBuilder {
            n_lanes: 1,
            sidewalks: true,
            parking: true,
            one_way: false,
        }
    }
}

impl LanePatternBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn n_lanes(mut self, n_lanes: u32) -> Self {
        assert!(n_lanes > 0);
        self.n_lanes = n_lanes;
        self
    }

    pub fn sidewalks(mut self, sidewalks: bool) -> Self {
        self.sidewalks = sidewalks;
        self
    }

    pub fn parking(mut self, parking: bool) -> Self {
        self.parking = parking;
        self
    }

    pub fn one_way(mut self, one_way: bool) -> Self {
        self.one_way = one_way;
        self
    }

    pub fn width(self) -> f32 {
        let mut w = 0.0;
        if self.sidewalks {
            w += LaneKind::Walking.width() * 2.0;
        }
        if self.parking {
            w += LaneKind::Parking.width() * 2.0;
        }
        w += self.n_lanes as f32 * 2.0 * LaneKind::Driving.width();
        w + 0.5
    }

    pub fn build(self) -> LanePattern {
        let mut backward = if self.one_way {
            vec![]
        } else {
            (0..self.n_lanes).map(|_| LaneKind::Driving).collect()
        };

        let mut forward: Vec<_> = (0..self.n_lanes).map(|_| LaneKind::Driving).collect();

        if self.parking {
            if !self.one_way {
                backward.push(LaneKind::Parking);
            }
            forward.push(LaneKind::Parking);
        }

        if self.sidewalks {
            backward.push(LaneKind::Walking);
            forward.push(LaneKind::Walking);
        }

        LanePattern {
            lanes_backward: backward,
            lanes_forward: forward,
        }
    }
}

impl Lane {
    pub fn make(
        parent: &mut Road,
        store: &mut Lanes,
        lane_type: LaneKind,
        direction: LaneDirection,
        dist_from_bottom: f32,
    ) -> LaneID {
        let (src, dst) = match direction {
            LaneDirection::Forward => (parent.src, parent.dst),
            LaneDirection::Backward => (parent.dst, parent.src),
        };

        store.insert_with_key(|id| Lane {
            id,
            parent: parent.id,
            src,
            dst,
            kind: lane_type,
            points: parent.points().clone(),
            width: lane_type.width(),
            dist_from_bottom,
            length: 0.0,
            control: TrafficControl::Always,
        })
    }

    pub fn get_inter_node_pos(&self, id: IntersectionID) -> Vec2 {
        match (id, self.points.as_slice()) {
            (x, [p, ..]) if x == self.src => *p,
            (x, [.., p]) if x == self.dst => *p,
            _ => panic!("Oh no"),
        }
    }

    pub fn gen_pos(&mut self, parent_road: &Road) {
        let dist_from_bottom = self.dist_from_bottom;
        let lane_dist = self.width * 0.5 + dist_from_bottom - parent_road.width * 0.5;

        let middle_points = parent_road.interfaced_points();

        let src_nor = -parent_road.src_dir().perpendicular();
        self.points
            .clear_push(middle_points.first() + src_nor * lane_dist);
        self.points.reserve(middle_points.n_points() - 1);

        for [a, elbow, c] in middle_points.array_windows::<3>() {
            let x = unwrap_contlog!((elbow - a).try_normalize(), "elbow too close to a");
            let y = unwrap_contlog!((elbow - c).try_normalize(), "elbow too close to c");

            let mut dir = (x + y).try_normalize().unwrap_or(-x.perpendicular());

            if x.perp_dot(y) < 0.0 {
                dir = -dir;
            }

            let mul = 1.0 + (1.0 + x.dot(y).min(0.0)) * (std::f32::consts::SQRT_2 - 1.0);

            let nor = mul * lane_dist * dir;
            self.points.push(elbow + nor);
        }

        let dst_nor = parent_road.dst_dir().perpendicular();
        self.points.push(middle_points.last() + dst_nor * lane_dist);

        if self.dir_from(parent_road.src) == TraverseDirection::Backward {
            self.points.reverse();
        }

        self.length = self.points.length();
    }

    pub fn control_point(&self) -> Vec2 {
        self.points.last()
    }

    pub fn proj(&self, p: Vec2) -> Vec2 {
        self.points.project(p)
    }

    pub fn dist2_to(&self, p: Vec2) -> f32 {
        self.points.project_dist2(p)
    }

    pub fn dir_from(&self, i: IntersectionID) -> TraverseDirection {
        if self.src == i {
            TraverseDirection::Forward
        } else {
            TraverseDirection::Backward
        }
    }

    pub fn orientation_from(&self, id: IntersectionID) -> Vec2 {
        if id == self.src {
            self.points.first_dir().unwrap_or(Vec2::UNIT_X)
        } else {
            -self.points.last_dir().unwrap_or(Vec2::UNIT_X)
        }
    }
}
