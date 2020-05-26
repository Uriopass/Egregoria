use crate::geometry::polyline::PolyLine;
use crate::geometry::Vec2;
use crate::map_model::{
    Intersection, IntersectionID, Intersections, Road, TrafficControl, TraverseDirection,
};
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LaneID;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaneDirection {
    Forward,
    Backward,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lane {
    pub id: LaneID,
    pub kind: LaneKind,

    pub control: TrafficControl,

    /// Src and dst implies direction
    pub src: IntersectionID,
    pub dst: IntersectionID,

    /// Always from src to dst
    pub points: PolyLine,
    pub width: f32,

    /// Length from intersection to intersection
    pub inter_length: f32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanePattern {
    pub lanes_forward: Vec<LaneKind>,
    pub lanes_backward: Vec<LaneKind>,
}

#[derive(Clone, Copy, Inspect)]
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

    pub fn build(self) -> LanePattern {
        let mut backward = if self.one_way {
            vec![]
        } else {
            (0..self.n_lanes).map(|_| LaneKind::Driving).collect()
        };

        let mut forward: Vec<_> = (0..self.n_lanes).map(|_| LaneKind::Driving).collect();

        if self.parking {
            backward.push(LaneKind::Parking);
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
    pub fn get_inter_node_pos(&self, id: IntersectionID) -> Vec2 {
        match (id, self.points.as_slice()) {
            (x, [p, ..]) if x == self.src => *p,
            (x, [.., p]) if x == self.dst => *p,
            _ => panic!("Oh no"),
        }
    }

    fn get_node_pos(
        &self,
        inter: &Intersection,
        parent_road: &Road,
        dist_from_bottom: f32,
    ) -> Vec2 {
        let lane_dist = self.width / 2.0 + dist_from_bottom - parent_road.width / 2.0;

        let dir = parent_road.orientation_from(inter.id);
        let dir_normal: Vec2 = if inter.id == parent_road.src {
            [-dir.y, dir.x].into()
        } else {
            [dir.y, -dir.x].into()
        };

        let dist = parent_road.interface_from(inter.id);

        inter.pos + dir * dist + dir_normal * lane_dist
    }

    pub fn gen_pos(
        &mut self,
        intersections: &Intersections,
        parent_road: &Road,
        dist_from_bottom: f32,
    ) {
        let pos_src = self.get_node_pos(
            &intersections[parent_road.src],
            parent_road,
            dist_from_bottom,
        );
        let pos_dst = self.get_node_pos(
            &intersections[parent_road.dst],
            parent_road,
            dist_from_bottom,
        );

        self.points.clear();
        self.points.push(pos_src);

        for window in parent_road.interpolation_points().as_slice().windows(3) {
            let a = window[0];
            let elbow = window[1];
            let c = window[2];

            let (x, _): (Vec2, _) = unwrap_or!((elbow - a).dir_dist(), continue);
            let (y, _) = unwrap_or!((elbow - c).dir_dist(), continue);

            let (mut dir, _) = unwrap_or!((x + y).dir_dist(), continue);

            if x.perp_dot(y) < 0.0 {
                dir = -dir;
            }

            let mul = 1.0 + (1.0 + x.dot(y).min(0.0)) * (std::f32::consts::SQRT_2 - 1.0);

            let nor = mul * (dist_from_bottom - parent_road.width * 0.5 + self.width * 0.5) * dir;
            self.points.push(elbow + nor);
        }

        self.points.push(pos_dst);

        if self.dir_from(parent_road.src) == TraverseDirection::Backward {
            self.points.reverse();
        }

        self.inter_length = parent_road.length;
    }

    pub fn dist2_to(&self, p: Vec2) -> f32 {
        (self.points.project(p).unwrap() - p).magnitude2()
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
            self.points.begin_dir().unwrap()
        } else {
            -self.points.end_dir().unwrap()
        }
    }
}
