use crate::{
    Intersection, IntersectionID, Lane, LaneDirection, LaneID, LaneKind, LanePattern, Lanes, LotID,
    ParkingSpots, Roads, SpatialMap,
};
use geom::PolyLine;
use geom::Spline;
use geom::Vec2;
use geom::AABB;
use geom::OBB;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct RoadID;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

    pub segment: RoadSegmentKind,

    // always from src to dst
    pub points: PolyLine,
    pub width: f32,

    src_interface: f32,
    dst_interface: f32,

    lanes_forward: Vec<(LaneID, LaneKind)>,
    lanes_backward: Vec<(LaneID, LaneKind)>,

    pub(crate) lots: Vec<LotID>,
}
#[derive(Copy, Clone)]
pub struct LanePair {
    pub incoming: Option<LaneID>,
    pub outgoing: Option<LaneID>,
}

impl Road {
    /// Builds the road and its associated lanes
    pub fn make<'a>(
        src: &Intersection,
        dst: &Intersection,
        segment: RoadSegmentKind,
        lane_pattern: &LanePattern,
        roads: &'a mut Roads,
        lanes: &mut Lanes,
        parking: &mut ParkingSpots,
        spatial: &mut SpatialMap,
    ) -> &'a Road {
        let points = Self::generate_points(src, dst, segment);

        let id = roads.insert_with_key(|id| Self {
            id,
            src: src.id,
            dst: dst.id,
            src_interface: 9.0,
            dst_interface: 9.0,
            segment,
            width: lane_pattern.width(),
            lanes_forward: vec![],
            lanes_backward: vec![],
            points,
            lots: vec![],
        });
        #[allow(clippy::indexing_slicing)]
        let road = &mut roads[id];

        let mut dist_from_bottom = 0.0;
        for (lane_k, dir) in lane_pattern.lanes() {
            let id = Lane::make(road, lanes, lane_k, dir, dist_from_bottom);

            match dir {
                LaneDirection::Forward => road.lanes_forward.insert(0, (id, lane_k)),
                LaneDirection::Backward => road.lanes_backward.push((id, lane_k)),
            }

            dist_from_bottom += lane_k.width();
        }

        road.update_lanes(lanes, parking);

        spatial.insert(id, road.bbox());
        road
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
                .iter()
                .find(|(_, kind)| matches!(kind, LaneKind::Walking))
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

    pub fn update_lanes(&self, lanes: &mut Lanes, parking: &mut ParkingSpots) {
        for (id, _) in self.lanes_iter() {
            let l = unwrap_contlog!(lanes.get_mut(id), "lane in road does not exist anymore");
            l.gen_pos(self);
            if matches!(l.kind, LaneKind::Parking) {
                parking.generate_spots(l);
            }
        }
    }

    pub fn length(&self) -> f32 {
        self.points.length()
    }

    pub fn bbox(&self) -> AABB {
        self.points.bbox().expand(self.width * 0.5)
    }

    pub fn pattern(&self) -> LanePattern {
        LanePattern {
            lanes_forward: self.lanes_forward.iter().map(|&(_, kind)| kind).collect(),
            lanes_backward: self.lanes_backward.iter().map(|&(_, kind)| kind).collect(),
        }
    }

    pub fn points(&self) -> &PolyLine {
        &self.points
    }

    pub fn interfaced_points(&self) -> PolyLine {
        self.points()
            .cut(self.interface_from(self.src), self.interface_from(self.dst))
    }

    fn generate_points(
        src: &Intersection,
        dst: &Intersection,
        segment: RoadSegmentKind,
    ) -> PolyLine {
        let from = src.pos;
        let to = dst.pos;

        PolyLine::new(match segment {
            RoadSegmentKind::Straight => {
                vec![from, to]
            }
            RoadSegmentKind::Curved((from_derivative, to_derivative)) => {
                let mut poly = vec![];

                let s = Spline {
                    from: src.pos,
                    to: dst.pos,
                    from_derivative,
                    to_derivative,
                };

                let points = s.smart_points(1.0, s.project_t(from, 0.3), s.project_t(to, 0.3));
                poly.extend(points);

                poly
            }
        })
    }

    pub fn interface_point(&self, id: IntersectionID) -> Vec2 {
        if id == self.src {
            self.interfaced_points().first()
        } else if id == self.dst {
            self.interfaced_points().last()
        } else {
            panic!("Asking interface from an intersection not connected to the road");
        }
    }

    pub fn interface_from(&self, id: IntersectionID) -> f32 {
        let (my_interf, other_interf) = self.interfaces_from(id);

        let l = self.points.length() - 2.0;
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

    pub fn dir_from(&self, id: IntersectionID) -> Vec2 {
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

    pub fn intersects(&self, obb: &OBB) -> bool {
        let c = obb.center();
        self.points
            .project(c)
            .is_close(c, (self.width + obb.axis()[0].magnitude()) * 0.5)
            || obb
                .corners
                .iter()
                .any(|&p| self.points.project(p).is_close(p, self.width * 0.5))
    }

    pub fn src_dir(&self) -> Vec2 {
        self.points.first_dir().unwrap_or(Vec2::UNIT_X)
    }

    pub fn dst_dir(&self) -> Vec2 {
        -self.points.last_dir().unwrap_or(Vec2::UNIT_X)
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
