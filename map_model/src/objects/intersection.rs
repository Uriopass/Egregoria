use crate::{
    Intersections, LaneID, Lanes, LightPolicy, RoadID, Roads, SpatialMap, TraverseDirection, Turn,
    TurnID, TurnPolicy,
};
use geom::pseudo_angle;
use geom::Polygon;
use geom::Rect;
use geom::Spline;
use geom::Vec2;
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
            polygon: Default::default(),
        });
        spatial.insert(id, Rect::new(pos.x, pos.y, 0.0, 0.0));
        id
    }

    pub fn add_road(&mut self, road_id: RoadID, roads: &Roads) {
        self.roads.push(road_id);

        let id = self.id;
        self.roads
            .sort_by_key(|&x| OrderedFloat(pseudo_angle(roads[x].basic_orientation_from(id))));
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

    pub fn update_interface_radius(&self, roads: &mut Roads) {
        for &r in &self.roads {
            roads[r].set_interface(self.id, 9.0);
        }

        if self.roads.len() == 1 {
            return;
        }

        for i in 0..self.roads.len() {
            let r1_id = self.roads[i];
            let r2_id = self.roads[(i + 1) % self.roads.len()];

            let r1 = &roads[r1_id];
            let r2 = &roads[r2_id];

            let width1 = r1.width * 0.5;
            let width2 = r2.width * 0.5;

            let w = (width1.powi(2) + width2.powi(2)).sqrt();

            let dir1 = r1.basic_orientation_from(self.id);
            let dir2 = r2.basic_orientation_from(self.id);

            let ang = dir1.angle(dir2).abs();

            let min_dist = w * 1.1 / ang.max(0.2).min(std::f32::consts::FRAC_PI_2).sin();
            roads[r1_id].max_interface(self.id, min_dist);
            roads[r2_id].max_interface(self.id, min_dist);
        }
    }

    pub fn update_polygon(&mut self, roads: &Roads) {
        self.polygon.clear();

        for (i, &road) in self.roads.iter().enumerate() {
            let road = &roads[road];
            let next_road = &roads[self.roads[(i + 1) % self.roads.len()]];

            let src_orient = road.orientation_from(self.id);

            let left =
                road.interface_point(self.id) - road.width * 0.5 * src_orient.perpendicular();

            let dst_orient = next_road.orientation_from(self.id);
            let next_right = next_road.interface_point(self.id)
                + next_road.width * 0.5 * dst_orient.perpendicular();

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
    }

    pub fn neighbors<'a>(&'a self, roads: &'a Roads) -> impl Iterator<Item = IntersectionID> + 'a {
        self.roads.iter().map(move |&x| roads[x].other_end(self.id))
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

    pub fn turns(&self) -> &Vec<Turn> {
        &self.turns
    }
}
