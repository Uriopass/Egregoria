use crate::geometry::pseudo_angle;
use crate::geometry::Vec2;
use crate::gui::InspectDragf;
use crate::map_model::{
    Intersections, LaneID, Lanes, LightPolicy, RoadID, Roads, Turn, TurnID, TurnPolicy,
};
use crate::utils::Restrict;
use cgmath::{Angle, InnerSpace};
use imgui_inspect_derive::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use specs::storage::BTreeStorage;
use specs::Component;
use std::collections::BTreeMap;

new_key_type! {
    pub struct IntersectionID;
}

impl IntersectionID {
    pub fn as_ffi(self) -> u64 {
        self.0.as_ffi()
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Inspect)]
#[storage(BTreeStorage)]
pub struct IntersectionComponent {
    #[inspect(skip = true)]
    pub id: IntersectionID,
    #[inspect(proxy_type = "InspectDragf")]
    pub radius: f32,
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    pub pos: Vec2,
    pub barycenter: Vec2,

    pub turns: BTreeMap<TurnID, Turn>,

    // sorted by angle
    pub roads: Vec<RoadID>,

    pub interface_radius: f32,
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

impl Intersection {
    pub fn make(store: &mut Intersections, pos: Vec2) -> IntersectionID {
        store.insert_with_key(|id| Intersection {
            id,
            pos,
            barycenter: pos,
            turns: BTreeMap::new(),
            roads: vec![],
            interface_radius: 5.0,
            turn_policy: TurnPolicy::default(),
            light_policy: LightPolicy::default(),
        })
    }

    pub fn add_road(&mut self, road_id: RoadID, lanes: &mut Lanes, roads: &Roads) {
        self.roads.push(road_id);

        self.update_turns(lanes, roads);
        self.update_traffic_control(lanes, roads);
    }

    pub fn remove_road(&mut self, road_id: RoadID, lanes: &mut Lanes, roads: &Roads) {
        self.roads.retain(|x| *x != road_id);

        self.update_turns(lanes, roads);
        self.update_traffic_control(lanes, roads);
    }

    pub fn update_turns(&mut self, lanes: &Lanes, roads: &Roads) {
        let id = self.id;
        let pos = self.pos;
        self.roads
            .sort_by_key(|&x| OrderedFloat(pseudo_angle(roads[x].dir_from(id, pos))));

        let turns = self.turn_policy.generate_turns(self, lanes, roads);
        let to_remove: Vec<TurnID> = self
            .turns
            .iter_mut()
            .filter(|(id, _)| turns.iter().find(|(id2, _)| id2 == *id).is_none())
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            self.turns.remove(&id);
        }

        for (turn_id, kind) in turns {
            self.turns
                .entry(turn_id)
                .or_insert_with(|| Turn::new(turn_id, kind));
        }

        for turn in self.turns.values_mut() {
            turn.make_points(lanes);
        }
    }

    pub fn update_traffic_control(&self, lanes: &mut Lanes, roads: &Roads) {
        self.light_policy.apply(self, lanes, roads);
    }

    pub fn update_interface_radius(&mut self, lanes: &Lanes, roads: &Roads) {
        let mut max_dist: f32 = 10.0;
        if self.roads.len() == 1 {
            self.interface_radius = max_dist;
            return;
        }
        for i in 0..self.roads.len() {
            let r1 = &roads[self.roads[i]];
            let r2 = &roads[self.roads[(i + 1) % self.roads.len()]];

            let width1 = r1.max_dist(lanes);
            let width2 = r2.max_dist(lanes);

            let w = (width1.powi(2) + width2.powi(2)).sqrt();

            max_dist = max_dist.max(w);

            let dir1 = r1.dir_from(self.id, self.pos);
            let dir2 = r2.dir_from(self.id, self.pos);

            let ang = dir1.angle(dir2).normalize_signed().0.abs();

            max_dist = max_dist.max(w / ang.restrict(0.1, std::f32::consts::FRAC_PI_2).sin());
        }
        self.interface_radius = max_dist * 1.1;
    }

    pub fn update_barycenter(&mut self, lanes: &Lanes, roads: &Roads) {
        let mut n_lanes = 0;
        let mut barycenter = vec2!(0.0, 0.0);

        if self.roads.len() <= 1 {
            self.barycenter = self.pos;
            return;
        }

        for road_id in &self.roads {
            for lane_id in roads[*road_id].lanes_iter() {
                let lane = &lanes[*lane_id];
                barycenter += lane.get_inter_node_pos(self.id);
                n_lanes += 1;
            }
        }

        self.barycenter = if n_lanes == 0 {
            self.pos
        } else {
            barycenter / (n_lanes as f32)
        };
    }

    pub fn turns_from_iter(&self, lane: LaneID) -> impl Iterator<Item = &TurnID> {
        self.turns
            .iter()
            .filter(move |(id, _)| id.src == lane)
            .map(|(id, _)| id)
    }

    pub fn turns_from(&self, lane: LaneID) -> Vec<&Turn> {
        self.turns
            .iter()
            .filter(|(id, _)| id.src == lane)
            .map(|(_, x)| x)
            .collect()
    }

    pub fn turns_adirectional(&self, lane: LaneID) -> Vec<&Turn> {
        self.turns
            .iter()
            .filter(|(id, _)| id.src == lane || id.dst == lane)
            .map(|(_, x)| x)
            .collect()
    }
}
