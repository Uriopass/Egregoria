use crate::map_model::{
    Intersection, IntersectionID, Intersections, Lane, LaneDirection, LaneID, LanePattern,
    LaneType, Lanes, NavMesh, Roads,
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

    pub interpolation_points: Vec<Vector2<f32>>,

    pub lanes_forward: Vec<LaneID>,
    pub lanes_backward: Vec<LaneID>,

    pub pattern: LanePattern,
}

impl Road {
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
            interpolation_points: vec![pos_src, pos_dst],
            lanes_forward: vec![],
            lanes_backward: vec![],
            pattern: lane_pattern.clone(),
        });
        let road = &mut store[id];
        for lane in &lane_pattern.lanes_forward {
            road.add_lane(lanes, *lane, LaneDirection::Forward);
        }
        for lane in &lane_pattern.lanes_backward {
            road.add_lane(lanes, *lane, LaneDirection::Backward);
        }
        id
    }

    pub fn is_one_way(&self) -> bool {
        self.lanes_forward.is_empty() || self.lanes_backward.is_empty()
    }

    pub fn n_lanes(&self) -> usize {
        self.lanes_backward.len() + self.lanes_forward.len()
    }

    pub fn add_lane(
        &mut self,
        store: &mut Lanes,
        lane_type: LaneType,
        direction: LaneDirection,
    ) -> LaneID {
        let id = store.insert_with_key(|id| Lane {
            id,
            parent: self.id,
            src_i: self.src,
            dst_i: self.dst,
            lane_type,
            src_node: None,
            dst_node: None,
            direction,
        });
        match direction {
            LaneDirection::Forward => self.lanes_forward.push(id),
            LaneDirection::Backward => self.lanes_backward.push(id),
        };
        id
    }

    pub fn gen_navmesh(
        &mut self,
        intersections: &Intersections,
        lanes: &mut Lanes,
        navmesh: &mut NavMesh,
    ) {
        self.interpolation_points[0] = intersections[self.src].pos;
        let l = self.interpolation_points.len();
        self.interpolation_points[l - 1] = intersections[self.dst].pos;

        for lane in &self.lanes_forward {
            let lane = &mut lanes[*lane];
            lane.gen_navmesh(intersections, self, navmesh);
        }

        for lane in &self.lanes_backward {
            let lane = &mut lanes[*lane];
            lane.gen_navmesh(intersections, self, navmesh);
        }
    }

    pub fn dir_from(&self, i: &Intersection) -> Vector2<f32> {
        if i.id == self.src {
            (self.interpolation_points[1] - i.pos).normalize()
        } else if i.id == self.dst {
            (self.interpolation_points[self.interpolation_points.len() - 2] - i.pos).normalize()
        } else {
            panic!("Asking dir from from an intersection not conected to the road");
        }
    }

    pub fn incoming_lanes_from(&self, id: IntersectionID) -> &Vec<LaneID> {
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
        self.interpolation_points
            .windows(2)
            .map(|x| (x[0] - x[1]).magnitude())
            .sum()
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
            return x + 1;
        }
        if let Some((x, _)) = self
            .lanes_forward
            .iter()
            .enumerate()
            .find(|(_, x)| **x == lane)
        {
            return x + 1;
        }
        0
    }
}
