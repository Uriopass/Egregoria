use crate::{
    IntersectionID, Intersections, Lane, LaneDirection, LaneID, LaneKind, LanePattern, Lanes, Map,
    ParkingSpots,
};
use geom::polyline::PolyLine;
use geom::splines::Spline;
use geom::Vec2;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct RoadID;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RoadSegmentKind {
    Straight,
    Curved((Vec2, Vec2)), // The two derivatives for the spline
}

impl RoadSegmentKind {
    pub fn from_elbow(from: Vec2, to: Vec2, elbow: Vec2) -> RoadSegmentKind {
        RoadSegmentKind::Curved((
            (elbow - from) * std::f32::consts::FRAC_1_SQRT_2,
            (to - elbow) * std::f32::consts::FRAC_1_SQRT_2,
        ))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Road {
    pub id: RoadID,
    pub src: IntersectionID,
    pub dst: IntersectionID,

    pub src_point: Vec2,
    pub dst_point: Vec2,

    pub segment: RoadSegmentKind,

    pub(crate) generated_points: PolyLine,

    pub length: f32,
    pub width: f32,

    pub src_interface: f32,
    pub dst_interface: f32,

    lanes_forward: Vec<(LaneID, LaneKind)>,
    lanes_backward: Vec<(LaneID, LaneKind)>,
}

#[derive(Copy, Clone)]
pub struct LanePair {
    pub incoming: Option<LaneID>,
    pub outgoing: Option<LaneID>,
}

impl Road {
    /// Builds the road and its associated lanes
    pub fn make(
        src: IntersectionID,
        dst: IntersectionID,
        segment: RoadSegmentKind,
        lane_pattern: &LanePattern,
        map: &mut Map,
    ) -> RoadID {
        let intersections = &map.intersections;
        let id = map.roads.insert_with_key(|id| Self {
            id,
            src,
            dst,
            src_interface: 9.0,
            dst_interface: 9.0,
            src_point: intersections[src].pos,
            dst_point: intersections[dst].pos,
            segment,
            width: 0.0,
            length: 1.0,
            lanes_forward: vec![],
            lanes_backward: vec![],
            generated_points: unsafe { PolyLine::new_unchecked(vec![]) },
        });
        let road = &mut map.roads[id];
        for (lane_k, dir) in lane_pattern.lanes() {
            let id = Lane::make(road, &mut map.lanes, lane_k, dir);

            match dir {
                LaneDirection::Forward => &mut road.lanes_forward,
                LaneDirection::Backward => &mut road.lanes_backward,
            }
            .push((id, lane_k));

            road.width += lane_k.width();
        }
        road.gen_pos(intersections, &mut map.lanes, &mut map.parking);
        map.spatial_map.insert_road(road);
        id
    }

    pub fn is_one_way(&self) -> bool {
        self.lanes_forward.is_empty() || self.lanes_backward.is_empty()
    }

    pub fn n_lanes(&self) -> usize {
        self.lanes_backward.len() + self.lanes_forward.len()
    }

    pub fn lanes_iter(&self) -> impl Iterator<Item = (LaneID, LaneKind)> + '_ {
        self.lanes_forward
            .iter()
            .rev()
            .chain(self.lanes_backward.iter())
            .copied()
    }

    pub fn sidewalks(&self, from: IntersectionID) -> LanePair {
        self.mk_pair(from, |lanes| {
            lanes
                .last()
                .filter(|(_, kind)| matches!(kind, LaneKind::Walking))
                .map(|&(id, _)| id)
        })
    }

    pub fn parking_next_to(&self, lane: &Lane) -> Option<LaneID> {
        let lanes = if lane.src == self.src {
            &self.lanes_forward
        } else {
            &self.lanes_backward
        };

        lanes[(lanes.len() - 2).max(0)..]
            .iter()
            .find(|(_, kind)| matches!(kind, LaneKind::Parking))
            .map(|&(id, _)| id)
    }

    fn mk_pair(
        &self,
        from: IntersectionID,
        find: fn(&[(LaneID, LaneKind)]) -> Option<LaneID>,
    ) -> LanePair {
        let fw = find(&self.lanes_forward);
        let bw = find(&self.lanes_backward);

        if from == self.src {
            LanePair {
                incoming: bw,
                outgoing: fw,
            }
        } else {
            LanePair {
                incoming: fw,
                outgoing: bw,
            }
        }
    }

    pub fn gen_pos(
        &mut self,
        intersections: &Intersections,
        lanes: &mut Lanes,
        parking: &mut ParkingSpots,
    ) {
        self.src_point = intersections[self.src].pos;
        self.dst_point = intersections[self.dst].pos;

        self.generate_points();

        self.length = match self.segment {
            RoadSegmentKind::Straight => (self.dst_point - self.src_point).magnitude(),
            RoadSegmentKind::Curved((from_derivative, to_derivative)) => Spline {
                from: self.src_point,
                to: self.dst_point,
                from_derivative,
                to_derivative,
            }
            .length(1.0),
        };

        let mut dist_from_bottom = 0.0;
        for (id, kind) in self.lanes_iter() {
            let l = &mut lanes[id];
            l.gen_pos(self, dist_from_bottom);
            dist_from_bottom += l.width;
            if matches!(kind, LaneKind::Parking) {
                parking.generate_spots(l)
            }
        }
    }

    pub fn pattern(&self) -> LanePattern {
        LanePattern {
            lanes_forward: self.lanes_forward.iter().map(|&(_, kind)| kind).collect(),
            lanes_backward: self.lanes_backward.iter().map(|&(_, kind)| kind).collect(),
        }
    }

    pub fn generated_points(&self) -> &PolyLine {
        &self.generated_points
    }

    fn generate_points(&mut self) {
        let (src_dir, dst_dir) = self.basic_orientations();

        let from = self.src_point + src_dir * self.interface_from(self.src);
        let to = self.dst_point + dst_dir * self.interface_from(self.dst);

        match &self.segment {
            RoadSegmentKind::Straight => {
                self.generated_points.clear_push(from);
                self.generated_points.push(to);
            }
            &RoadSegmentKind::Curved((from_derivative, to_derivative)) => {
                let s = Spline {
                    from: self.src_point,
                    to: self.dst_point,
                    from_derivative,
                    to_derivative,
                };

                let mut points = s.smart_points(1.0, s.project_t(from, 0.3), s.project_t(to, 0.3));
                self.generated_points.clear_push(points.next().unwrap()); // unwrap ok: smart_points have start and end
                self.generated_points.extend(points);
            }
        }
    }

    pub fn interface_point(&self, id: IntersectionID) -> Vec2 {
        if id == self.src {
            self.generated_points.first()
        } else if id == self.dst {
            self.generated_points.last()
        } else {
            panic!("Asking interface from an intersection not connected to the road");
        }
    }

    pub fn interface_from(&self, id: IntersectionID) -> f32 {
        let (my_interf, other_interf) = self.interfaces_from(id);

        let l = self.length - 2.0;
        let half = l * 0.5;

        if my_interf + other_interf > l {
            if my_interf > half && other_interf > half {
                half
            } else if my_interf > half {
                l - other_interf
            } else {
                my_interf
            }
        } else {
            my_interf
        }
    }

    fn interfaces_from(&self, id: IntersectionID) -> (f32, f32) {
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

    pub fn basic_orientations(&self) -> (Vec2, Vec2) {
        match &self.segment {
            RoadSegmentKind::Straight => {
                let d = (self.dst_point - self.src_point).normalize();
                (d, -d)
            }
            RoadSegmentKind::Curved((s, d)) => (s.normalize(), -d.normalize()),
        }
    }

    pub fn basic_orientation_from(&self, id: IntersectionID) -> Vec2 {
        let (src_dir, dst_dir) = self.basic_orientations();

        if id == self.src {
            src_dir
        } else if id == self.dst {
            dst_dir
        } else {
            panic!("Asking dir from from an intersection not conected to the road");
        }
    }

    pub fn orientation_from(&self, id: IntersectionID) -> Vec2 {
        if id == self.src {
            self.src_dir()
        } else if id == self.dst {
            self.dst_dir()
        } else {
            panic!("Asking dir from from an intersection not conected to the road");
        }
    }

    pub fn incoming_lanes_to(&self, id: IntersectionID) -> &Vec<(LaneID, LaneKind)> {
        if id == self.src {
            &self.lanes_backward
        } else if id == self.dst {
            &self.lanes_forward
        } else {
            panic!("Asking incoming lanes from from an intersection not conected to the road");
        }
    }

    pub fn outgoing_lanes_from(&self, id: IntersectionID) -> &Vec<(LaneID, LaneKind)> {
        if id == self.src {
            &self.lanes_forward
        } else if id == self.dst {
            &self.lanes_backward
        } else {
            panic!("Asking outgoing lanes from from an intersection not conected to the road");
        }
    }

    pub fn project(&self, p: Vec2) -> Vec2 {
        self.generated_points.project(p)
    }

    pub fn src_dir(&self) -> Vec2 {
        self.generated_points.first_dir().unwrap_or(Vec2::UNIT_X)
    }

    pub fn dst_dir(&self) -> Vec2 {
        -self.generated_points.last_dir().unwrap_or(Vec2::UNIT_X)
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
