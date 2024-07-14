use crate::map::{IntersectionID, Lanes, Road, RoadID, TrafficControl, TraverseDirection};
use egui_inspect::Inspect;
use geom::{PolyLine3, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;

new_key_type! {
    pub struct LaneID;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[repr(u8)]
pub enum LaneKind {
    Driving,
    Biking,
    Bus,
    Parking,
    Walking,
    Rail,
}

impl LaneKind {
    #[inline]
    pub fn vehicles(self) -> bool {
        matches!(self, LaneKind::Driving | LaneKind::Biking | LaneKind::Bus)
    }

    #[inline]
    pub fn needs_light(self) -> bool {
        matches!(self, LaneKind::Driving | LaneKind::Biking | LaneKind::Bus)
    }

    #[inline]
    pub fn needs_arrows(self) -> bool {
        matches!(
            self,
            LaneKind::Driving | LaneKind::Biking | LaneKind::Bus | LaneKind::Rail
        )
    }

    #[inline]
    pub fn is_rail(self) -> bool {
        matches!(self, LaneKind::Rail)
    }

    #[inline]
    pub const fn width(self) -> f32 {
        match self {
            LaneKind::Driving | LaneKind::Biking | LaneKind::Bus => 4.0,
            LaneKind::Parking => 2.5,
            LaneKind::Walking => 3.0,
            LaneKind::Rail => 5.3,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaneDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lane {
    pub id: LaneID,
    pub parent: RoadID,

    /// Src and dst implies direction
    pub src: IntersectionID,
    pub dst: IntersectionID,

    pub kind: LaneKind,

    pub control: TrafficControl,
    pub speed_limit: f32,

    /// Always from src to dst
    pub points: PolyLine3,
    pub dist_from_bottom: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LanePattern {
    pub lanes_forward: Vec<(LaneKind, f32)>,
    pub lanes_backward: Vec<(LaneKind, f32)>,
}

impl LanePattern {
    pub fn lanes(&self) -> impl Iterator<Item = (LaneKind, LaneDirection, f32)> + '_ {
        self.lanes_forward
            .iter()
            .rev()
            .map(|&(k, limit)| (k, LaneDirection::Forward, limit))
            .chain(
                self.lanes_backward
                    .iter()
                    .map(|&(k, limit)| (k, LaneDirection::Backward, limit)),
            )
    }

    pub fn width(&self) -> f32 {
        self.lanes().map(|(kind, _, _)| kind.width()).sum()
    }
}

#[derive(PartialEq, Copy, Clone, Inspect)]
pub struct LanePatternBuilder {
    pub n_lanes: u32,
    #[inspect(name = "speed", step = 1.0, min_value = 4.0, max_value = 40.0)]
    pub speed_limit: f32,
    pub sidewalks: bool,
    pub parking: bool,
    pub one_way: bool,
    pub rail: bool,
}
impl Eq for LanePatternBuilder {}

impl Default for LanePatternBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LanePatternBuilder {
    pub const fn new() -> Self {
        LanePatternBuilder {
            n_lanes: 1,
            speed_limit: 9.0,
            sidewalks: true,
            parking: true,
            one_way: false,
            rail: false,
        }
    }

    pub const fn n_lanes(mut self, n_lanes: u32) -> Self {
        self.n_lanes = if n_lanes > 10 { 10 } else { n_lanes };
        self
    }

    pub const fn sidewalks(mut self, sidewalks: bool) -> Self {
        self.sidewalks = sidewalks;
        self
    }

    pub const fn speed_limit(mut self, limit: f32) -> Self {
        self.speed_limit = limit;
        self
    }

    pub const fn parking(mut self, parking: bool) -> Self {
        self.parking = parking;
        self
    }

    pub const fn one_way(mut self, one_way: bool) -> Self {
        self.one_way = one_way;
        self
    }

    pub const fn rail(mut self, rail: bool) -> Self {
        self.rail = rail;
        self
    }

    pub fn width(self) -> f32 {
        if self.rail {
            let wayf = if self.one_way { 1.0 } else { 2.0 };
            return self.n_lanes as f32 * LaneKind::Rail.width() * wayf;
        }

        let mut w = 0.0;
        let wayf = if self.one_way { 1.0 } else { 2.0 };
        if self.sidewalks {
            w += LaneKind::Walking.width() * 2.0;
        }
        if self.parking {
            w += LaneKind::Parking.width() * wayf;
        }
        w += self.n_lanes as f32 * wayf * LaneKind::Driving.width();
        w + 0.5
    }

    pub fn build(mut self) -> LanePattern {
        if self.n_lanes == 0 {
            self.parking = false;
            self.sidewalks = true;
        }

        let mut backward = if self.one_way {
            vec![]
        } else {
            (0..self.n_lanes).map(|_| LaneKind::Driving).collect()
        };

        let mut forward: Vec<_> = (0..self.n_lanes).map(|_| LaneKind::Driving).collect();

        if self.parking {
            if !self.one_way {
                backward.push(LaneKind::Parking);
            }
            forward.push(LaneKind::Parking);
        }

        if self.sidewalks {
            backward.push(LaneKind::Walking);
            forward.push(LaneKind::Walking);
        }

        if self.rail {
            backward = if self.one_way {
                vec![]
            } else {
                (0..self.n_lanes).map(|_| LaneKind::Rail).collect()
            };
            forward = (0..self.n_lanes).map(|_| LaneKind::Rail).collect();
        }

        LanePattern {
            lanes_backward: backward
                .into_iter()
                .map(|x| (x, self.speed_limit))
                .collect(),
            lanes_forward: forward.into_iter().map(|x| (x, self.speed_limit)).collect(),
        }
    }
}

impl Lane {
    pub fn make(
        parent: &mut Road,
        store: &mut Lanes,
        kind: LaneKind,
        speed_limit: f32,
        direction: LaneDirection,
        dist_from_bottom: f32,
    ) -> LaneID {
        let (src, dst) = match direction {
            LaneDirection::Forward => (parent.src, parent.dst),
            LaneDirection::Backward => (parent.dst, parent.src),
        };

        store.insert_with_key(|id| Lane {
            id,
            parent: parent.id,
            src,
            dst,
            kind,
            points: parent.points().clone(),
            dist_from_bottom,
            control: TrafficControl::Always,
            speed_limit,
        })
    }

    pub fn get_inter_node_pos(&self, id: IntersectionID) -> Vec3 {
        match (id, self.points.as_slice()) {
            (x, [p, ..]) if x == self.src => *p,
            (x, [.., p]) if x == self.dst => *p,
            _ => panic!("Oh no"),
        }
    }

    pub fn gen_pos(&mut self, parent_road: &Road) {
        let dist_from_bottom = self.dist_from_bottom;
        let lane_dist = self.kind.width() * 0.5 + dist_from_bottom - parent_road.width * 0.5;

        let middle_points = parent_road.interfaced_points();

        let src_nor = -unwrap_retlog!(
            middle_points.first_dir(),
            "not enough points in interfaced points"
        )
        .perp_up();
        self.points
            .clear_push(middle_points.first() + src_nor * lane_dist);
        self.points.reserve(middle_points.n_points() - 1);

        for [a, elbow, c] in middle_points.array_windows::<3>() {
            let x = unwrap_contlog!((elbow - a).xy().try_normalize(), "elbow too close to a");
            let y = unwrap_contlog!((elbow - c).xy().try_normalize(), "elbow too close to c");

            let dir = match (x + y).try_normalize() {
                Some(v) => {
                    let d = x.perp_dot(y);
                    if d.abs() < 0.01 {
                        -x.perpendicular()
                    } else if d < 0.0 {
                        -v
                    } else {
                        v
                    }
                }
                None => -x.perpendicular(),
            };

            let mul = 1.0 + (1.0 + x.dot(y).min(0.0)) * (std::f32::consts::SQRT_2 - 1.0);

            let nor = mul * lane_dist * dir;
            self.points.push(elbow + nor.z0());
        }

        let dst_nor = -unwrap_retlog!(
            middle_points.last_dir(),
            "not enough points in interfaced points"
        )
        .perp_up();
        self.points.push(middle_points.last() + dst_nor * lane_dist);

        if self.dst == parent_road.src {
            self.points.reverse();
        }
    }

    pub fn control_point(&self) -> Vec3 {
        self.points.last()
    }

    pub fn dir_from(&self, i: IntersectionID) -> TraverseDirection {
        if self.src == i {
            TraverseDirection::Forward
        } else {
            TraverseDirection::Backward
        }
    }

    /// Returns the vector pointing to the lane from the intersection center
    pub fn orientation_from(&self, id: IntersectionID) -> Vec2 {
        if id == self.src {
            self.points.first_dir().unwrap_or(Vec3::X).xy()
        } else {
            -self.points.last_dir().unwrap_or(Vec3::X).xy()
        }
    }
}

debug_inspect_impl!(LaneID);
