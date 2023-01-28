use crate::map::{
    Intersection, IntersectionID, Lane, LaneDirection, LaneID, LaneKind, LanePattern, Lanes,
    ParkingSpots, Roads, SpatialMap, Terrain,
};
use geom::Spline3;
use geom::{BoldLine, PolyLine3};
use geom::{Vec2, Vec3};
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
    // don't try to make points go away from the road as it would be impossible to split them correctly afterward
    pub points: PolyLine3,
    pub interfaced_points: PolyLine3,
    pub width: f32,

    src_interface: f32,
    dst_interface: f32,

    lanes_forward: Vec<(LaneID, LaneKind)>,
    lanes_backward: Vec<(LaneID, LaneKind)>,
}
#[derive(Copy, Clone)]
pub struct LanePair {
    pub incoming: Option<LaneID>,
    pub outgoing: Option<LaneID>,
}

pub struct PylonPosition {
    pub terrain_height: f32,
    pub pos: Vec3,
    pub dir: Vec3,
}

impl Road {
    /// Builds the road and its associated lanes
    pub fn make(
        src: &Intersection,
        dst: &Intersection,
        segment: RoadSegmentKind,
        lane_pattern: &LanePattern,
        roads: &mut Roads,
        lanes: &mut Lanes,
        parking: &mut ParkingSpots,
        spatial: &mut SpatialMap,
    ) -> RoadID {
        let width = lane_pattern.width();
        let points = Self::generate_points(
            src,
            dst,
            segment,
            lane_pattern.lanes().any(|(a, _, _)| a.is_rail()),
        );

        let id = roads.insert_with_key(|id| Self {
            id,
            src: src.id,
            dst: dst.id,
            src_interface: 9.0,
            dst_interface: 9.0,
            segment,
            width,
            lanes_forward: vec![],
            lanes_backward: vec![],
            interfaced_points: PolyLine3::new(vec![points.first()]),
            points,
        });
        #[allow(clippy::indexing_slicing)]
        let road = &mut roads[id];

        let mut dist_from_bottom = 0.0;
        for (lane_k, dir, limit) in lane_pattern.lanes() {
            let id = Lane::make(road, lanes, lane_k, limit, dir, dist_from_bottom);

            match dir {
                LaneDirection::Forward => road.lanes_forward.insert(0, (id, lane_k)),
                LaneDirection::Backward => road.lanes_backward.push((id, lane_k)),
            }

            dist_from_bottom += lane_k.width();
        }

        road.update_lanes(lanes, parking);

        spatial.insert(id, road.boldline());
        road.id
    }

    pub fn is_one_way(&self) -> bool {
        self.lanes_forward.is_empty() || self.lanes_backward.is_empty()
    }

    pub fn n_lanes(&self) -> usize {
        self.lanes_backward.len() + self.lanes_forward.len()
    }

    /// Returns lanes in left to right order from the source
    pub fn lanes_iter(&self) -> impl DoubleEndedIterator<Item = (LaneID, LaneKind)> + '_ {
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

        lanes
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

    pub fn update_lanes(&mut self, lanes: &mut Lanes, parking: &mut ParkingSpots) {
        self.update_interfaced_points();
        for (id, _) in self.lanes_iter() {
            let l = unwrap_contlog!(lanes.get_mut(id), "lane in road does not exist anymore");
            l.gen_pos(self);
            if matches!(l.kind, LaneKind::Parking) {
                parking.generate_spots(l);
            }
        }
        parking.clean_reuse();
    }

    pub fn length(&self) -> f32 {
        self.points.length()
    }

    pub fn boldline(&self) -> BoldLine {
        BoldLine::new(self.points.flatten(), self.width * 0.5)
    }

    pub fn pattern(&self, lanes: &Lanes) -> LanePattern {
        LanePattern {
            lanes_forward: self
                .lanes_forward
                .iter()
                .flat_map(|&(id, kind)| {
                    Some((
                        kind,
                        unwrap_or!(lanes.get(id), {
                            log::error!("lane doesn't exist while gettign pattern");
                            return None;
                        })
                        .speed_limit,
                    ))
                })
                .collect(),
            lanes_backward: self
                .lanes_backward
                .iter()
                .flat_map(|&(id, kind)| {
                    Some((
                        kind,
                        unwrap_or!(lanes.get(id), {
                            log::error!("lane doesn't exist while gettign pattern");
                            return None;
                        })
                        .speed_limit,
                    ))
                })
                .collect(),
        }
    }

    pub fn pylons_positions<'a>(
        interfaced_points: &'a PolyLine3,
        terrain: &'a Terrain,
    ) -> impl Iterator<Item = PylonPosition> + 'a {
        interfaced_points
            .equipoints_dir(80.0, true)
            .filter_map(move |(pos, dir)| {
                let h = terrain.height(pos.xy())?;
                if (h - pos.z).abs() <= 2.0 {
                    return None;
                }
                Some(PylonPosition {
                    terrain_height: h,
                    pos,
                    dir,
                })
            })
    }

    pub fn points(&self) -> &PolyLine3 {
        &self.points
    }
    pub fn interfaced_points(&self) -> &PolyLine3 {
        &self.interfaced_points
    }

    fn update_interfaced_points(&mut self) {
        let points = &self.points;
        self.interfaced_points =
            points.cut(self.interface_from(self.src), self.interface_from(self.dst));

        let cpoints = &mut self.interfaced_points;
        let o_beg = points.first().z;
        let o_end = points.last().z;
        let i_beg = cpoints.first().z;
        let i_end = cpoints.last().z;
        let i_range = i_end - i_beg;
        let o_range = o_end - o_beg;

        if i_range.abs() < 0.01 {
            if cpoints.n_points() == 1 {
                return;
            }

            let n = cpoints.n_points() as f32 - 1.0;
            for (i, v) in cpoints.iter_mut().enumerate() {
                v.z = o_beg + (i as f32) * o_range / n;
            }
            return;
        }

        for v in cpoints.iter_mut() {
            v.z = ((v.z - i_beg) * o_range) / i_range + o_beg;
        }
    }

    fn generate_points(
        src: &Intersection,
        dst: &Intersection,
        segment: RoadSegmentKind,
        precise: bool,
    ) -> PolyLine3 {
        let from = src.pos;
        let to = dst.pos;
        let diff = to - from;

        let spline = match segment {
            RoadSegmentKind::Straight if diff.z.abs() > 0.5 => Spline3 {
                from,
                to,
                from_derivative: (diff * 0.3).xy().z0(),
                to_derivative: (diff * 0.3).xy().z0(),
            },
            RoadSegmentKind::Straight => {
                return PolyLine3::new(vec![from, to]);
            }
            RoadSegmentKind::Curved((from_derivative, to_derivative)) => Spline3 {
                from,
                to,
                from_derivative: from_derivative.z0(),
                to_derivative: to_derivative.z0(),
            },
        };

        let mut iter = spline.smart_points(if precise { 0.1 } else { 1.0 }, 0.0, 1.0);
        let mut p = PolyLine3::new(vec![iter.next().unwrap()]);

        for v in iter {
            if v == to {
                p.push(v);
                break;
            }
            if v.is_close(from, 1.0) {
                continue;
            }
            if v.is_close(to, 1.0) {
                continue;
            }
            p.push(v);
        }

        p
    }

    pub fn interface_point(&self, id: IntersectionID) -> Vec3 {
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
            panic!("Asking interface from from an intersection not connected to the road");
        }
    }

    pub fn set_interface(&mut self, id: IntersectionID, v: f32) {
        if id == self.src {
            self.src_interface = v;
        } else if id == self.dst {
            self.dst_interface = v;
        } else {
            panic!("Setting interface from from an intersection not connected to the road");
        }
    }

    pub fn max_interface(&mut self, id: IntersectionID, v: f32) {
        if id == self.src {
            self.src_interface = self.src_interface.max(v);
        } else if id == self.dst {
            self.dst_interface = self.dst_interface.max(v);
        } else {
            panic!("Setting interface from from an intersection not connected to the road");
        }
    }

    pub fn dir_from(&self, id: IntersectionID) -> Vec2 {
        if id == self.src {
            self.src_dir()
        } else if id == self.dst {
            self.dst_dir()
        } else {
            panic!("Asking dir from from an intersection not connected to the road");
        }
    }

    pub fn incoming_lanes_to(&self, id: IntersectionID) -> &Vec<(LaneID, LaneKind)> {
        if id == self.src {
            &self.lanes_backward
        } else if id == self.dst {
            &self.lanes_forward
        } else {
            panic!("Asking incoming lanes from from an intersection not connected to the road");
        }
    }

    pub fn outgoing_lanes_from(&self, id: IntersectionID) -> &Vec<(LaneID, LaneKind)> {
        if id == self.src {
            &self.lanes_forward
        } else if id == self.dst {
            &self.lanes_backward
        } else {
            panic!("Asking outgoing lanes from from an intersection not connected to the road");
        }
    }

    pub fn src_dir(&self) -> Vec2 {
        self.points.first_dir().unwrap_or(Vec3::X).xy().normalize()
    }

    pub fn dst_dir(&self) -> Vec2 {
        -self.points.last_dir().unwrap_or(Vec3::X).xy().normalize()
    }

    pub fn other_end(&self, my_end: IntersectionID) -> Option<IntersectionID> {
        if self.src == my_end {
            return Some(self.dst);
        }
        if self.dst == my_end {
            return Some(self.src);
        }
        None
    }
}
