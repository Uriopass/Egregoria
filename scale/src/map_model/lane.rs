use crate::geometry::polyline::PolyLine;
use crate::geometry::segment::Segment;
use crate::map_model::{IntersectionID, Intersections, Road, RoadID, TrafficControl};
use cgmath::InnerSpace;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct LaneID;
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LaneKind {
    Driving,
    Biking,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaneDirection {
    Forward,
    Backward,
}

#[derive(Serialize, Deserialize)]
pub struct Lane {
    pub id: LaneID,
    pub parent: RoadID,
    pub kind: LaneKind,

    pub control: TrafficControl,

    pub src: IntersectionID,
    pub dst: IntersectionID,

    // Always from start to finish. (depends on direction)
    pub points: PolyLine,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LanePattern {
    pub name: String,
    pub lanes_forward: Vec<LaneKind>,
    pub lanes_backward: Vec<LaneKind>,
}

impl LanePattern {
    pub fn one_way(n_lanes: usize) -> Self {
        assert!(n_lanes > 0);
        LanePattern {
            lanes_backward: vec![LaneKind::Walking],
            lanes_forward: (0..n_lanes)
                .map(|_| LaneKind::Driving)
                .chain(std::iter::once(LaneKind::Walking))
                .collect(),
            name: if n_lanes == 1 {
                "One way".to_owned()
            } else {
                format!("One way {} lanes", n_lanes)
            },
        }
    }

    pub fn two_way(n_lanes: usize) -> Self {
        assert!(n_lanes > 0);
        LanePattern {
            lanes_backward: (0..n_lanes)
                .map(|_| LaneKind::Driving)
                .chain(std::iter::once(LaneKind::Walking))
                .collect(),
            lanes_forward: (0..n_lanes)
                .map(|_| LaneKind::Driving)
                .chain(std::iter::once(LaneKind::Walking))
                .collect(),
            name: if n_lanes == 1 {
                "Two way".to_owned()
            } else {
                format!("Two way {} lanes", n_lanes)
            },
        }
    }
}

impl Lane {
    pub fn get_inter_node_pos(&self, id: IntersectionID) -> Vector2<f32> {
        match (id, self.points.as_slice()) {
            (x, [p, ..]) if x == self.src => *p,
            (x, [.., p]) if x == self.dst => *p,
            _ => panic!("Oh no"),
        }
    }

    fn get_node_pos(
        &self,
        inter_id: IntersectionID,
        inters: &Intersections,
        parent_road: &Road,
    ) -> Vector2<f32> {
        let inter = &inters[inter_id];

        let lane_dist = 0.5 + parent_road.idx_unchecked(self.id) as f32;
        let dir = parent_road.dir_from(inter_id, inter.pos);
        let dir_normal: Vector2<f32> = if inter_id == self.dst {
            [-dir.y, dir.x].into()
        } else {
            [dir.y, -dir.x].into()
        };

        let mindist = parent_road.length() / 2.0 - 1.0;

        inter.pos + dir * inter.interface_radius.min(mindist) + dir_normal * lane_dist as f32 * 8.0
    }

    pub fn gen_pos(&mut self, intersections: &Intersections, parent_road: &Road) {
        let pos_src = self.get_node_pos(self.src, intersections, parent_road);
        let pos_dst = self.get_node_pos(self.dst, intersections, parent_road);

        self.points.clear();
        self.points.push(pos_src);
        self.points.push(pos_dst);
    }

    pub fn dist_to(&self, p: Vector2<f32>) -> f32 {
        let segm = Segment::new(self.points[0], self.points[1]);
        (segm.project(p) - p).magnitude()
    }

    pub fn get_orientation_vec(&self) -> Vector2<f32> {
        let src = self.points[0];
        let dst = self.points[1];

        assert_ne!(dst, src);

        (dst - src).normalize()
    }
}
