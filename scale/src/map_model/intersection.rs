use crate::geometry::pseudo_angle;
use crate::geometry::Vec2;
use crate::gui::InspectDragf;
use crate::interaction::{Movable, Selectable};
use crate::map_model::{
    Intersections, LaneID, Lanes, LightPolicy, RoadID, Roads, Turn, TurnID, TurnPolicy,
};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::Color;
use imgui_inspect_derive::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use specs::storage::BTreeStorage;
use specs::{Builder, Component, Entities, Entity, LazyUpdate};
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

pub fn make_inter_entity<'a>(
    inter: &Intersection,
    inter_pos: Vec2,
    lazy: &LazyUpdate,
    entities: &Entities<'a>,
) -> Entity {
    lazy.create_entity(entities)
        .with(IntersectionComponent {
            id: inter.id,
            radius: inter.interface_radius,
            turn_policy: inter.turn_policy,
            light_policy: inter.light_policy,
        })
        .with(MeshRender::simple(
            CircleRender {
                radius: 2.0,
                color: Color {
                    a: 0.5,
                    ..Color::BLUE
                },
                filled: true,
                ..CircleRender::default()
            },
            2,
        ))
        .with(Transform::new(inter_pos))
        .with(Movable)
        .with(Selectable::new(10.0))
        .build()
}
