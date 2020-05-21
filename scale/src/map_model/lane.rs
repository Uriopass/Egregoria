use crate::geometry::polyline::PolyLine;
use crate::geometry::Vec2;
use crate::map_model::{
    Intersection, IntersectionID, Intersections, Road, TrafficControl, TraverseDirection,
};
use cgmath::InnerSpace;
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

    pub src: IntersectionID,
    pub dst: IntersectionID,

    /// Always from start to finish. (depends on direction)
    pub points: PolyLine,
    pub width: f32,

    /// Length from intersection to intersection
    pub inter_length: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LanePattern {
    pub name: String,
    pub lanes_forward: Vec<LaneKind>,
    pub lanes_backward: Vec<LaneKind>,
}

impl PartialEq for LanePattern {
    fn eq(&self, other: &Self) -> bool {
        self.lanes_backward == other.lanes_backward && self.lanes_forward == other.lanes_forward
    }
}

impl Eq for LanePattern {}

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

        let mut name = if self.one_way { "One way" } else { "Two way" }.to_owned();
        name.push_str(&format!(" {} lanes", self.n_lanes));

        if !self.sidewalks {
            name.push_str(&" no sidewalks");
        }
        LanePattern {
            lanes_backward: backward,
            lanes_forward: forward,
            name,
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

        let dir = parent_road.dir_from(inter.id);
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
        let pos_src = self.get_node_pos(&intersections[self.src], parent_road, dist_from_bottom);
        let pos_dst = self.get_node_pos(&intersections[self.dst], parent_road, dist_from_bottom);

        self.points.clear();
        self.points.push(pos_src);
        self.points.push(pos_dst);

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

    pub fn get_orientation_vec(&self) -> Vec2 {
        let src = self.points[0];
        let dst = self.points[1];

        assert_ne!(dst, src);

        (dst - src).normalize()
    }
}
