use crate::geometry::pseudo_angle;
use crate::geometry::Vec2;
use crate::map_model::{
    Intersections, LaneID, Lanes, LightPolicy, RoadID, Roads, TraverseDirection, Turn, TurnID,
    TurnPolicy,
};
use crate::utils::Restrict;
use cgmath::{Angle, InnerSpace};
use imgui_inspect_derive::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use specs::storage::BTreeStorage;
use specs::Component;

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
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    pub pos: Vec2,
    pub barycenter: Vec2,

    turns: Vec<Turn>,

    // sorted by angle
    pub roads: Vec<RoadID>,

    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

impl Intersection {
    pub fn make(store: &mut Intersections, pos: Vec2) -> IntersectionID {
        store.insert_with_key(|id| Intersection {
            id,
            pos,
            barycenter: pos,
            turns: Default::default(),
            roads: vec![],
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

        self.turns
            .retain(|t| turns.iter().any(|(id2, _)| *id2 == t.id));

        for (turn_id, kind) in turns {
            if self.turns.iter().all(|x| x.id != turn_id) {
                self.turns.push(Turn::new(turn_id, kind))
            }
        }

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

            let width1 = r1.width / 2.0;
            let width2 = r2.width / 2.0;

            let w = (width1.powi(2) + width2.powi(2)).sqrt();

            let dir1 = r1.dir_from(self.id, self.pos);
            let dir2 = r2.dir_from(self.id, self.pos);

            let ang = dir1.angle(dir2).normalize_signed().0.abs();

            let min_dist = w * 1.1 / ang.restrict(0.1, std::f32::consts::FRAC_PI_2).sin();
            roads[r1_id].max_interface(self.id, min_dist);
            roads[r2_id].max_interface(self.id, min_dist);
        }
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
