use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;

use geom::{lerp, BoldLine, Degrees, PolyLine3};
use geom::{PolyLine, Spline3};
use geom::{Vec2, Vec3};

use crate::map::{
    Environment, Intersection, IntersectionID, Lane, LaneDirection, LaneID, LaneKind, LanePattern,
    Lanes, ParkingSpots, Roads, SpatialMap, ROAD_Z_OFFSET,
};

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

pub enum PointGenerateError {
    OutsideOfMap,
    TooSteep,
}

impl Road {
    /// Builds the road and its associated lanes
    pub fn make(
        src: &Intersection,
        dst: &Intersection,
        segment: RoadSegmentKind,
        lane_pattern: &LanePattern,
        env: &Environment,
        roads: &mut Roads,
        lanes: &mut Lanes,
        parking: &mut ParkingSpots,
        spatial: &mut SpatialMap,
    ) -> Option<RoadID> {
        let width = lane_pattern.width();
        let points = Self::generate_points(
            src.pos,
            dst.pos,
            segment,
            lane_pattern.lanes().any(|(a, _, _)| a.is_rail()),
            env,
        )
        .ok()?;

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
        Some(road.id)
    }

    pub fn is_one_way(&self) -> bool {
        self.lanes_forward.is_empty() || self.lanes_backward.is_empty()
    }

    pub fn n_lanes(&self) -> usize {
        self.lanes_backward.len() + self.lanes_forward.len()
    }

    /// Returns lanes in left to right order from the source
    pub fn lanes_iter(&self) -> impl DoubleEndedIterator<Item = (LaneID, LaneKind)> + Clone + '_ {
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

    pub fn has_sidewalks(&self) -> bool {
        self.lanes_forward
            .iter()
            .any(|(_, kind)| matches!(kind, LaneKind::Walking))
            || self
                .lanes_backward
                .iter()
                .any(|(_, kind)| matches!(kind, LaneKind::Walking))
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
        env: &'a Environment,
    ) -> impl Iterator<Item = PylonPosition> + 'a {
        interfaced_points
            .equipoints_dir(80.0, true)
            .filter_map(move |(pos, dir)| {
                let h = env.height(pos.xy())?;
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

        let start = cpoints.first().clone();
        let end = cpoints.last().clone();

        for v in cpoints.iter_mut_unchecked() {
            let start_coeff = v.distance(start) / 5.0;
            v.z = lerp(i_beg, v.z, start_coeff.clamp(0.0, 1.0));

            let end_coeff = v.distance(end) / 5.0;
            v.z = lerp(i_end, v.z, end_coeff.clamp(0.0, 1.0));
        }

        cpoints.recalculate_length();
    }

    // Run an algorithm to find the height of the road at each point
    // This is not easy because the terrain can take many shapes
    // The algorithm is as follow:
    // - First compute the terrain contour every meter
    // - Then find out which points are airborn (according to maxslope)
    // - Then find the interface points where points become airborn
    // - Then linear interpolate the points between the interface points
    // - Then smooth out the result to avoid huge derivative changes
    // - Then simplify the result to avoid too many points
    //
    // maxslope is the maximum meter of height difference per meter of distance (1.0 is a 45° slope)
    pub fn heightfinder(
        p: &PolyLine,
        start_height: f32,
        end_height: f32,
        maxslope: f32,
        env: &Environment,
    ) -> Result<PolyLine3, PointGenerateError> {
        // first calculate the contour

        let mut contour = Vec::with_capacity(p.length() as usize + 2);
        let mut points = Vec::with_capacity(contour.len());

        for pos in std::iter::once(p.first())
            .chain(
                p.points_dirs_along((1..p.length() as u32).map(|v| v as f32))
                    .map(|v| v.0),
            )
            .chain(std::iter::once(p.last()))
        {
            let h = env.height(pos).ok_or(PointGenerateError::OutsideOfMap)?;
            contour.push(h);
            points.push(pos.z(h));
        }

        contour[0] = start_height;
        *contour.last_mut().unwrap() = end_height;

        // Then find out which points are airborn (according to maxslope)
        // To do that, we do two passes (one forward, one backward) to find the airborn points

        let mut airborn = Vec::with_capacity(contour.len());

        let mut cur_height = contour[0];
        for &h in &contour {
            let diff = cur_height - h;

            airborn.push(diff > maxslope);
            cur_height -= diff.min(maxslope);
        }

        let mut cur_height = contour.last().copied().unwrap();
        let mut i = airborn.len();
        for &h in contour.iter().rev() {
            i -= 1;
            let diff = cur_height - h;

            airborn[i] |= diff > maxslope;
            cur_height -= diff.min(maxslope);
        }

        // Then find the interface points where points become airborn
        // To do that, we just find the points where airborn changes

        *airborn.first_mut().unwrap() = false;
        *airborn.last_mut().unwrap() = false;

        let mut interface = Vec::with_capacity(airborn.len());
        for i in 1..airborn.len() - 1 {
            if airborn[i] && !airborn[i - 1] {
                interface.push(i - 1);
            }
            if airborn[i] && !airborn[i + 1] {
                interface.push(i + 1);
            }
        }

        // Then linear interpolate the points between the interface points using a nice cubic

        fn cubic(t: f32) -> f32 {
            t * t * (3.0 - 2.0 * t)
        }

        for w in interface.windows(2) {
            let i1 = w[0];
            let i2 = w[1];

            let h1 = contour[i1];
            let h2 = contour[i2];

            let di = (i2 - i1) as f32;
            let dh = h2 - h1;

            for i in i1..=i2 {
                let coeff = (i - i1) as f32 / di;
                let h = h1 + dh * cubic(coeff);
                contour[i] = h;
            }
        }

        // Then smooth out the result to avoid huge derivative changes
        //let mut smoothed = vec![0.0; contour.len()];
        //smoothed[0] = contour[0];
        //let l = smoothed.len();
        //smoothed[l - 1] = contour[contour.len() - 1];
        //// must be odd to have the result in contour
        //for _ in 0..3 {
        //    for i in 1..smoothed.len() - 1 {
        //        smoothed[i] = (contour[i - 1] + contour[i] + contour[i + 1]) / 3.0;
        //    }
        //    std::mem::swap(&mut smoothed, &mut contour);
        //}

        for (h, v) in contour.into_iter().zip(points.iter_mut()) {
            v.z = h + ROAD_Z_OFFSET;
        }

        // Then simplify the result to avoid too many points
        let mut points = PolyLine3::new(points);
        points.simplify(Degrees(1.0).into(), 1.0, 100.0);

        Ok(points)
    }

    pub fn generate_points(
        from: Vec3,
        to: Vec3,
        segment: RoadSegmentKind,
        precise: bool,
        env: &Environment,
    ) -> Result<PolyLine3, PointGenerateError> {
        let spline = match segment {
            RoadSegmentKind::Straight => {
                let p = PolyLine::new(vec![from.xy(), to.xy()]);
                return Self::heightfinder(&p, from.z, to.z, 0.25, env);
            }
            RoadSegmentKind::Curved((from_derivative, to_derivative)) => Spline3 {
                from: from.up(ROAD_Z_OFFSET),
                to: to.up(ROAD_Z_OFFSET),
                from_derivative: from_derivative.z0(),
                to_derivative: to_derivative.z0(),
            },
        };

        let iter = spline.smart_points(if precise { 0.1 } else { 1.0 }, 0.0, 1.0);
        let mut p = PolyLine3::new(vec![from.up(ROAD_Z_OFFSET)]);

        for v in iter {
            if v.is_close(from, 1.0) {
                continue;
            }
            if v.is_close(to, 1.0) {
                continue;
            }
            p.push(v);
        }
        p.push(to.up(ROAD_Z_OFFSET));

        Ok(p)
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
