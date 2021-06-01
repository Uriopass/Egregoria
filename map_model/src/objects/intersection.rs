use crate::{
    Intersections, LaneID, LaneKind, Lanes, LightPolicy, Road, RoadID, Roads, SpatialMap,
    TraverseDirection, Turn, TurnID, TurnPolicy,
};
use geom::{pseudo_angle, Circle};
use geom::{Vec2, Vec3};
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
    pub pos: Vec3,

    turns: Vec<Turn>,

    // sorted by angle
    pub roads: Vec<RoadID>,

    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

impl Intersection {
    pub fn make(store: &mut Intersections, spatial: &mut SpatialMap, pos: Vec3) -> IntersectionID {
        let id = store.insert_with_key(|id| Intersection {
            id,
            pos,
            turns: Default::default(),
            roads: Default::default(),
            turn_policy: Default::default(),
            light_policy: Default::default(),
        });
        spatial.insert(id, pos.xy());
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
            center: self.pos.xy(),
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

    const MIN_INTERFACE: f32 = 9.0;
    // allow slicing since we remove all roads not in self.roads
    #[allow(clippy::indexing_slicing)]
    pub fn update_interface_radius(&mut self, roads: &mut Roads) {
        let id = self.id;
        self.check_dead_roads(roads);

        for &r in &self.roads {
            let r = &mut roads[r];
            r.set_interface(id, Self::empty_interface(r.width));
        }

        if self.roads.len() <= 1 {
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

    pub fn empty_interface(width: f32) -> f32 {
        (width * 0.8).max(Self::MIN_INTERFACE)
    }

    pub fn interface_at(&self, roads: &Roads, width: f32, dir: Vec2) -> f32 {
        let mut max_inter = Self::empty_interface(width);
        let id = self.id;
        for i in 0..self.roads.len() {
            let r1_id = self.roads[i];
            let r1 = &roads[r1_id];

            let width1 = r1.width * 0.5;
            let w = width1.hypot(width);
            let dir1 = r1.dir_from(id);

            let d = dir1.dot(dir).max(0.0).min(1.0);
            let sin = (1.0 - d * d).sqrt();

            let min_dist = w * 1.1 / sin;
            max_inter = max_inter.max(min_dist);
        }
        max_inter
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
