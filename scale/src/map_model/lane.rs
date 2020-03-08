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
pub enum LaneType {
    Driving,
    Biking,
    Bus,
    Construction,
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
    pub lane_type: LaneType,

    pub control: TrafficControl,

    pub src_i: IntersectionID,
    pub dst_i: IntersectionID,

    // Always from start to finish. (depends on direction)
    pub points: Vec<Vector2<f32>>,
    pub direction: LaneDirection,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LanePattern {
    pub name: String,
    pub lanes_forward: Vec<LaneType>,
    pub lanes_backward: Vec<LaneType>,
}
impl PartialEq for LanePattern {
    fn eq(&self, other: &Self) -> bool {
        self.lanes_forward == other.lanes_forward && self.lanes_backward == other.lanes_backward
    }
}
impl Eq for LanePattern {}

impl LanePattern {
    pub fn one_way(n_lanes: usize) -> Self {
        assert!(n_lanes > 0);
        LanePattern {
            lanes_backward: vec![],
            lanes_forward: (0..n_lanes).map(|_| LaneType::Driving).collect(),
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
            lanes_backward: (0..n_lanes).map(|_| LaneType::Driving).collect(),
            lanes_forward: (0..n_lanes).map(|_| LaneType::Driving).collect(),
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
        match (self.direction, id) {
            (LaneDirection::Forward, x) if x == self.src_i => self.points[0],
            (LaneDirection::Forward, x) if x == self.dst_i => *self.points.last().unwrap(),
            (LaneDirection::Backward, x) if x == self.src_i => *self.points.last().unwrap(),
            (LaneDirection::Backward, x) if x == self.dst_i => self.points[0],
            _ => panic!("Oh no"),
        }
    }

    fn get_node_pos(
        &self,
        inter_id: IntersectionID,
        incoming: bool,
        inters: &Intersections,
        parent_road: &Road,
    ) -> Vector2<f32> {
        let inter = &inters[inter_id];

        let mut lane_dist = 0.5 + parent_road.idx_unchecked(self.id) as f32;
        let dir = parent_road.dir_from(inter);
        let dir_normal: Vector2<f32> = if incoming {
            [-dir.y, dir.x].into()
        } else {
            [dir.y, -dir.x].into()
        };

        if parent_road.is_one_way() {
            lane_dist -= parent_road.n_lanes() as f32 / 2.0;
        }

        let mindist = parent_road.length() / 2.0 - 1.0;

        inter.pos + dir * inter.interface_radius.min(mindist) + dir_normal * lane_dist as f32 * 8.0
    }

    pub fn gen_pos(&mut self, intersections: &Intersections, parent_road: &Road) {
        let pos_src = self.get_node_pos(
            self.src_i,
            self.direction == LaneDirection::Backward,
            intersections,
            parent_road,
        );

        let pos_dst = self.get_node_pos(
            self.dst_i,
            self.direction == LaneDirection::Forward,
            intersections,
            parent_road,
        );

        self.points.clear();
        match self.direction {
            LaneDirection::Forward => {
                self.points.push(pos_src);
                self.points.push(pos_dst);
            }
            LaneDirection::Backward => {
                self.points.push(pos_dst);
                self.points.push(pos_src);
            }
        }
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

    pub fn forward_dst_inter(&self) -> IntersectionID {
        match self.direction {
            LaneDirection::Forward => self.dst_i,
            LaneDirection::Backward => self.src_i,
        }
    }
}
