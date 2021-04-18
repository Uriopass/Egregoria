use crate::{
    Intersections, LaneID, LaneKind, Lanes, LightPolicy, Road, RoadID, Roads, SpatialMap,
    TraverseDirection, Turn, TurnID, TurnPolicy,
};
use geom::Polygon;
use geom::Spline;
use geom::Vec2;
use geom::{pseudo_angle, Circle};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

new_key_type! {
    pub struct IntersectionID;
}

impl IntersectionID {
    pub fn as_ffi(self) -> u64 {
        self.0.as_ffi()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    pub pos: Vec2,

    turns: Vec<Turn>,

    // sorted by angle
    pub roads: Vec<RoadID>,

    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,

    pub polygon: Polygon,
}

impl Intersection {
    pub fn make(store: &mut Intersections, spatial: &mut SpatialMap, pos: Vec2) -> IntersectionID {
        let id = store.insert_with_key(|id| Intersection {
            id,
            pos,
            turns: Default::default(),
            roads: Default::default(),
            turn_policy: Default::default(),
            light_policy: Default::default(),
            polygon: Polygon::centered_rect(pos, 5.0, 5.0),
        });
        spatial.insert(id, pos);
        id
    }

    pub fn add_road(&mut self, roads: &Roads, road: &Road) {
        self.roads.push(road.id);

        let id = self.id;
        self.roads.retain(|&id| roads.contains_key(id));
        self.roads.sort_by_key(|&road| {
            #[allow(clippy::indexing_slicing)]
            OrderedFloat(pseudo_angle(roads[road].dir_from(id)))
        });
    }

    pub fn bcircle(&self, roads: &Roads) -> Circle {
        Circle {
            center: self.pos,
            radius: self
                .roads
                .iter()
                .flat_map(|x| roads.get(*x))
                .map(|x| OrderedFloat(x.interface_from(self.id)))
                .max()
                .map(|x| x.0)
                .unwrap_or(10.0),
        }
    }

    pub fn remove_road(&mut self, road_id: RoadID) {
        self.roads.retain(|x| *x != road_id);
    }

    pub fn update_turns(&mut self, lanes: &Lanes, roads: &Roads) {
        self.turns = self
            .turn_policy
            .generate_turns(self, lanes, roads)
            .into_iter()
            .map(|(id, kind)| Turn::new(id, kind))
            .collect();

        for turn in self.turns.iter_mut() {
            turn.make_points(lanes);
        }
    }

    pub fn update_traffic_control(&self, lanes: &mut Lanes, roads: &Roads) {
        self.light_policy.apply(self, lanes, roads);
    }

    fn check_dead_roads(&mut self, roads: &Roads) {
        let id = self.id;
        self.roads.retain(|x| {
            let v = roads.contains_key(*x);
            if !v {
                log::error!(
                    "{:?} contained unexisting {:?} when updating interface radius",
                    id,
                    x
                );
            }
            v
        });
    }

    // allow slicing since we remove all roads not in self.roads
    #[allow(clippy::indexing_slicing)]
    pub fn update_interface_radius(&mut self, roads: &mut Roads) {
        let id = self.id;
        self.check_dead_roads(roads);

        for &r in &self.roads {
            roads[r].set_interface(id, 9.0);
        }

        if let [ref r] = *self.roads {
            let r = &mut roads[*r];
            r.max_interface(id, r.width * 0.5);
            return;
        }

        for i in 0..self.roads.len() {
            let r1_id = self.roads[i];
            let r2_id = self.roads[(i + 1) % self.roads.len()];

            let r1 = &roads[r1_id];
            let r2 = &roads[r2_id];

            let width1 = r1.width * 0.5;
            let width2 = r2.width * 0.5;

            let w = width1.hypot(width2);

            let dir1 = r1.dir_from(id);
            let dir2 = r2.dir_from(id);

            let d = dir1.dot(dir2).max(0.0).min(1.0);
            let sin = (1.0 - d * d).sqrt();

            let min_dist = w * 1.1 / sin;
            roads[r1_id].max_interface(id, min_dist);
            roads[r2_id].max_interface(id, min_dist);
        }
    }

    pub fn update_polygon(&mut self, roads: &Roads) {
        self.polygon.clear();
        self.check_dead_roads(roads);

        for (i, &road) in self.roads.iter().enumerate() {
            #[allow(clippy::indexing_slicing)]
            let road = &roads[road];

            #[allow(clippy::indexing_slicing)]
            let next_road = &roads[self.roads[(i + 1) % self.roads.len()]];

            let mut fp = road.interfaced_points();

            if road.dst == self.id {
                fp.reverse();
            }

            let src_orient = unwrap_cont!(fp.first_dir());
            let left = fp.first() - src_orient.perpendicular() * road.width * 0.5;

            let mut fp = next_road.interfaced_points();

            if next_road.dst == self.id {
                fp.reverse();
            }

            let dst_orient = unwrap_cont!(fp.first_dir());
            let next_right = fp.first() + dst_orient.perpendicular() * next_road.width * 0.5;

            let ang = (-src_orient).angle(dst_orient);

            const TURN_ANG_ADD: f32 = 0.29;
            const TURN_ANG_MUL: f32 = 0.36;
            const TURN_MUL: f32 = 0.46;

            let dist = (next_right - left).magnitude()
                * (TURN_ANG_ADD + ang.abs() * TURN_ANG_MUL)
                * TURN_MUL;

            let spline = Spline {
                from: left,
                to: next_right,
                from_derivative: -src_orient * dist,
                to_derivative: dst_orient * dist,
            };

            self.polygon.extend(spline.smart_points(1.0, 0.0, 1.0));
        }

        if self.polygon.is_empty() {
            self.polygon = Polygon::centered_rect(self.pos, 5.0, 5.0);
        }
    }

    pub fn undirected_neighbors<'a>(
        &'a self,
        roads: &'a Roads,
    ) -> impl Iterator<Item = IntersectionID> + 'a {
        self.roads
            .iter()
            .flat_map(move |&x| roads.get(x).and_then(|r| r.other_end(self.id)))
    }

    pub fn driving_neighbours<'a>(
        &'a self,
        roads: &'a Roads,
    ) -> impl Iterator<Item = IntersectionID> + 'a {
        let id = self.id;
        self.roads.iter().flat_map(move |&x| {
            let r = roads.get(x)?;
            r.outgoing_lanes_from(id)
                .iter()
                .find(|(_, kind)| matches!(kind, LaneKind::Driving))?;
            r.other_end(id)
        })
    }

    pub fn find_turn(&self, needle: TurnID) -> Option<&Turn> {
        self.turns
            .iter()
            .find_map(move |x| if x.id == needle { Some(x) } else { None })
    }

    pub fn turns_from(
        &self,
        lane: LaneID,
    ) -> impl Iterator<Item = (TurnID, TraverseDirection)> + '_ {
        self.turns.iter().filter_map(move |Turn { id, .. }| {
            if id.src == lane {
                Some((*id, TraverseDirection::Forward))
            } else if id.bidirectional && id.dst == lane {
                Some((*id, TraverseDirection::Backward))
            } else {
                None
            }
        })
    }

    pub fn turns_to(&self, lane: LaneID) -> impl Iterator<Item = (TurnID, TraverseDirection)> + '_ {
        self.turns.iter().filter_map(move |Turn { id, .. }| {
            if id.dst == lane {
                Some((*id, TraverseDirection::Forward))
            } else if id.bidirectional && id.src == lane {
                Some((*id, TraverseDirection::Backward))
            } else {
                None
            }
        })
    }

    pub fn turns(&self) -> &Vec<Turn> {
        &self.turns
    }
}
