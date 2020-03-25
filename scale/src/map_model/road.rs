use crate::geometry::polyline::PolyLine;
use crate::map_model::{
    IntersectionID, Intersections, Lane, LaneDirection, LaneID, LaneKind, LanePattern, Lanes,
    Roads, TrafficControl,
};
use cgmath::InnerSpace;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct RoadID;
}

#[derive(Serialize, Deserialize)]
pub struct Road {
    pub id: RoadID,
    pub src: IntersectionID,
    pub dst: IntersectionID,

    pub interpolation_points: PolyLine,

    lanes_forward: Vec<LaneID>,
    lanes_backward: Vec<LaneID>,
}

impl Road {
    /// Builds the road and its associated lanes
    pub fn make(
        store: &mut Roads,
        intersections: &Intersections,
        src: IntersectionID,
        dst: IntersectionID,
        lanes: &mut Lanes,
        lane_pattern: &LanePattern,
    ) -> RoadID {
        let pos_src = intersections[src].pos;
        let pos_dst = intersections[dst].pos;

        debug_assert_ne!(pos_src, pos_dst);
        let id = store.insert_with_key(|id| Self {
            id,
            src,
            dst,
            interpolation_points: vec![pos_src, pos_dst].into(),
            lanes_forward: vec![],
            lanes_backward: vec![],
        });
        let road = &mut store[id];
        for lane in &lane_pattern.lanes_forward {
            road.add_lane(lanes, *lane, LaneDirection::Forward);
        }
        for lane in &lane_pattern.lanes_backward {
            road.add_lane(lanes, *lane, LaneDirection::Backward);
        }
        road.gen_pos(intersections, lanes);
        id
    }

    pub fn is_one_way(&self) -> bool {
        self.lanes_forward.is_empty() || self.lanes_backward.is_empty()
    }

    pub fn n_lanes(&self) -> usize {
        self.lanes_backward.len() + self.lanes_forward.len()
    }

    pub fn lanes_iter(&self) -> impl Iterator<Item = &LaneID> {
        self.lanes_forward.iter().chain(self.lanes_backward.iter())
    }

    pub fn sidewalk_forward(&self) -> Option<&LaneID> {
        self.lanes_forward.last()
    }

    pub fn add_lane(
        &mut self,
        store: &mut Lanes,
        lane_type: LaneKind,
        direction: LaneDirection,
    ) -> LaneID {
        let (src, dst) = match direction {
            LaneDirection::Forward => (self.src, self.dst),
            LaneDirection::Backward => (self.dst, self.src),
        };
        let id = store.insert_with_key(|id| Lane {
            id,
            parent: self.id,
            src,
            dst,
            control: TrafficControl::Always,
            kind: lane_type,
            points: Default::default(),
        });
        match direction {
            LaneDirection::Forward => self.lanes_forward.push(id),
            LaneDirection::Backward => self.lanes_backward.push(id),
        };
        id
    }

    pub fn gen_pos(&mut self, intersections: &Intersections, lanes: &mut Lanes) {
        self.interpolation_points.0[0] = intersections[self.src].pos;
        let l = self.interpolation_points.n_points();
        self.interpolation_points.0[l - 1] = intersections[self.dst].pos;

        for id in self.lanes_forward.iter().chain(self.lanes_backward.iter()) {
            lanes[*id].gen_pos(intersections, self);
        }
    }

    pub fn dir_from(&self, id: IntersectionID, pos: Vector2<f32>) -> Vector2<f32> {
        if id == self.src {
            (self.interpolation_points.0[1] - pos).normalize()
        } else if id == self.dst {
            (self.interpolation_points.0[self.interpolation_points.n_points() - 2] - pos)
                .normalize()
        } else {
            panic!("Asking dir from from an intersection not conected to the road");
        }
    }

    pub fn incoming_lanes_to(&self, id: IntersectionID) -> &Vec<LaneID> {
        if id == self.src {
            &self.lanes_backward
        } else if id == self.dst {
            &self.lanes_forward
        } else {
            panic!("Asking incoming lanes from from an intersection not conected to the road");
        }
    }

    pub fn outgoing_lanes_from(&self, id: IntersectionID) -> &Vec<LaneID> {
        if id == self.src {
            &self.lanes_forward
        } else if id == self.dst {
            &self.lanes_backward
        } else {
            panic!("Asking outgoing lanes from from an intersection not conected to the road");
        }
    }

    pub fn length(&self) -> f32 {
        self.interpolation_points.length()
    }

    pub fn other_end(&self, my_end: IntersectionID) -> IntersectionID {
        if self.src == my_end {
            return self.dst;
        } else if self.dst == my_end {
            return self.src;
        }
        panic!(
            "Asking other end of {:?} which isn't connected to {:?}",
            self.id, my_end
        );
    }

    pub fn idx_unchecked(&self, lane: LaneID) -> usize {
        if let Some((x, _)) = self
            .lanes_backward
            .iter()
            .enumerate()
            .find(|(_, x)| **x == lane)
        {
            return x;
        }
        if let Some((x, _)) = self
            .lanes_forward
            .iter()
            .enumerate()
            .find(|(_, x)| **x == lane)
        {
            return x;
        }
        0
    }
}
