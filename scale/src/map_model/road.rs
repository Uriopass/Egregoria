use crate::geometry::polyline::PolyLine;
use crate::geometry::Vec2;
use crate::map_model::{
    IntersectionID, Intersections, Lane, LaneDirection, LaneID, LaneKind, LanePattern, Lanes,
    Roads, TrafficControl,
};
use cgmath::InnerSpace;
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

    interpolation_points: PolyLine,
    pub length: f32,
    pub width: f32,

    pub src_interface: f32,
    pub dst_interface: f32,

    lanes_forward: Vec<LaneID>,
    lanes_backward: Vec<LaneID>,

    pub lane_pattern: LanePattern,
}

impl Road {
    /// Builds the road and its associated lanes
    pub fn make(
        store: &mut Roads,
        intersections: &Intersections,
        src: IntersectionID,
        dst: IntersectionID,
        lanes: &mut Lanes,
        lane_pattern: LanePattern,
    ) -> RoadID {
        let pos_src = intersections[src].pos;
        let pos_dst = intersections[dst].pos;

        debug_assert_ne!(pos_src, pos_dst);

        let points = PolyLine::new(vec![pos_src, pos_dst]);
        let id = store.insert_with_key(|id| Self {
            id,
            src,
            dst,
            src_interface: 9.0,
            dst_interface: 9.0,
            interpolation_points: points,
            width: 1.0,
            length: 1.0,
            lanes_forward: vec![],
            lanes_backward: vec![],
            lane_pattern: lane_pattern.clone(),
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
        self.lanes_forward
            .iter()
            .rev()
            .chain(self.lanes_backward.iter())
    }

    pub fn sidewalks<'a>(
        &self,
        from: IntersectionID,
        lanes: &'a Lanes,
    ) -> (Option<&'a Lane>, Option<&'a Lane>) {
        (
            self.incoming_lanes_to(from)
                .iter()
                .map(|x| &lanes[*x])
                .find(|x| matches!(x.kind, LaneKind::Walking)),
            self.outgoing_lanes_from(from)
                .iter()
                .map(|x| &lanes[*x])
                .find(|x| matches!(x.kind, LaneKind::Walking)),
        )
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

        let self_id = self.id;
        let id = store.insert_with_key(|id| Lane {
            id,
            parent: self_id,
            src,
            dst,
            control: TrafficControl::Always,
            kind: lane_type,
            points: Default::default(),
            width: if lane_type.vehicles() { 8.0 } else { 4.0 },
            parent_length: self.length,
        });
        match direction {
            LaneDirection::Forward => self.lanes_forward.push(id),
            LaneDirection::Backward => self.lanes_backward.push(id),
        }
        id
    }

    pub fn gen_pos(&mut self, intersections: &Intersections, lanes: &mut Lanes) {
        *self.interpolation_points.first_mut().unwrap() = intersections[self.src].pos;
        *self.interpolation_points.last_mut().unwrap() = intersections[self.dst].pos;
        self.length = self.interpolation_points.length();
        self.width = self.lanes_iter().map(|&x| lanes[x].width).sum();

        let mut dist_from_bottom = 0.0;
        for &id in self.lanes_iter() {
            let l = &mut lanes[id];
            l.gen_pos(intersections, self, dist_from_bottom);
            dist_from_bottom += l.width;
        }
    }

    pub fn interfaces_from(&self, id: IntersectionID) -> (f32, f32) {
        if id == self.src {
            (self.src_interface, self.dst_interface)
        } else if id == self.dst {
            (self.dst_interface, self.src_interface)
        } else {
            panic!("Asking interface from from an intersection not conected to the road");
        }
    }

    pub fn set_interface(&mut self, id: IntersectionID, v: f32) {
        if id == self.src {
            self.src_interface = v;
        } else if id == self.dst {
            self.dst_interface = v;
        } else {
            panic!("Setting interface from from an intersection not conected to the road");
        }
    }

    pub fn max_interface(&mut self, id: IntersectionID, v: f32) {
        if id == self.src {
            self.src_interface = self.src_interface.max(v);
        } else if id == self.dst {
            self.dst_interface = self.dst_interface.max(v);
        } else {
            panic!("Setting interface from from an intersection not conected to the road");
        }
    }

    pub fn dir_from(&self, id: IntersectionID, pos: Vec2) -> Vec2 {
        if id == self.src {
            (self.interpolation_points[1] - pos).normalize()
        } else if id == self.dst {
            (self.interpolation_points[self.interpolation_points.n_points() - 2] - pos).normalize()
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

    pub fn interpolation_points(&self) -> &PolyLine {
        &self.interpolation_points
    }

    pub fn src_point(&self) -> Vec2 {
        self.interpolation_points.first().unwrap()
    }

    pub fn dst_point(&self) -> Vec2 {
        self.interpolation_points.last().unwrap()
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
}
