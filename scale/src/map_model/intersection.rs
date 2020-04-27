use crate::geometry::pseudo_angle;
use crate::geometry::Vec2;
use crate::gui::InspectDragf;
use crate::map_model::{
    Intersections, LaneID, Lanes, LightPolicy, RoadID, Roads, Turn, TurnID, TurnPolicy,
};
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
            turns: BTreeMap::new(),
            roads: vec![],
            interface_radius: 20.0,
            turn_policy: TurnPolicy::default(),
            light_policy: LightPolicy::default(),
        })
    }

    pub fn remove_road(&mut self, road_id: RoadID, lanes: &mut Lanes, roads: &Roads) {
        self.roads.retain(|x| *x != road_id);

        self.gen_turns(lanes, roads);
        self.update_traffic_control(lanes, roads);
    }

    pub fn get_barycenter(&self, roads: &Roads, lanes: &Lanes) -> Vec2 {
        let mut n_lanes = 0;
        let mut barycenter = vec2!(0.0, 0.0);

        for road_id in &self.roads {
            for lane_id in roads[*road_id].lanes_iter() {
                let lane = &lanes[*lane_id];
                barycenter += lane.get_inter_node_pos(self.id);
                n_lanes += 1;
            }
        }

        if n_lanes == 0 {
            self.pos
        } else {
            barycenter / (n_lanes as f32)
        }
    }

    pub fn gen_turns(&mut self, lanes: &Lanes, roads: &Roads) {
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

    pub fn add_road(&mut self, road_id: RoadID, lanes: &mut Lanes, roads: &Roads) {
        self.roads.push(road_id);
        let id = self.id;
        let pos = self.pos;
        self.roads
            .sort_by_key(|&x| OrderedFloat(pseudo_angle(roads[x].dir_from(id, pos))));

        self.gen_turns(lanes, roads);
        self.update_traffic_control(lanes, roads);
    }

    pub fn update_traffic_control(&self, lanes: &mut Lanes, roads: &Roads) {
        self.light_policy.apply(self, lanes, roads);
    }
}
