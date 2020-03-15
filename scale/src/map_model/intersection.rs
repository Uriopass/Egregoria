use crate::gui::InspectDragf;
use crate::interaction::{Movable, Selectable};
use crate::map_model::{
    Intersections, LaneID, Lanes, LightPolicy, Road, RoadID, Roads, Turn, TurnID, TurnPolicy,
};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::{Color, BLUE};
use cgmath::Vector2;
use imgui_inspect_derive::*;
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
    pub pos: Vector2<f32>,

    pub turns: BTreeMap<TurnID, Turn>,

    pub incoming_lanes: Vec<LaneID>,
    pub outgoing_lanes: Vec<LaneID>,

    pub roads: Vec<RoadID>,

    pub interface_radius: f32,
    pub turn_policy: TurnPolicy,
    pub light_policy: LightPolicy,
}

impl Intersection {
    pub fn make(store: &mut Intersections, pos: Vector2<f32>) -> IntersectionID {
        store.insert_with_key(|id| Intersection {
            id,
            pos,
            turns: BTreeMap::new(),
            incoming_lanes: vec![],
            outgoing_lanes: vec![],
            roads: vec![],
            interface_radius: 20.0,
            turn_policy: TurnPolicy::default(),
            light_policy: LightPolicy::default(),
        })
    }

    pub fn clean(&mut self, lanes: &Lanes, roads: &Roads) {
        self.incoming_lanes.retain(|x| lanes.contains_key(*x));
        self.outgoing_lanes.retain(|x| lanes.contains_key(*x));

        self.roads.retain(|x| roads.contains_key(*x));

        self.gen_turns(lanes, roads);
    }

    pub fn gen_turns(&mut self, lanes: &Lanes, roads: &Roads) {
        let turns = self.turn_policy.generate_turns(self, lanes, roads);

        let to_remove: Vec<TurnID> = self
            .turns
            .iter_mut()
            .filter(|(id, _)| !turns.contains(id))
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            self.turns.remove(&id);
        }

        for turn in turns {
            self.turns.entry(turn).or_insert_with(|| Turn::new(turn));
        }

        for turn in self.turns.values_mut() {
            turn.make_points(lanes);
        }
    }

    pub fn add_road(&mut self, road: &Road) {
        self.roads.push(road.id);
        if road.src == self.id {
            self.fill_lanes(road.lanes_backward.clone(), road.lanes_forward.clone());
        } else if road.dst == self.id {
            self.fill_lanes(road.lanes_forward.clone(), road.lanes_backward.clone());
        } else {
            panic!(
                "Trying to add {:?} to {:?} but it's between {:?} and {:?}",
                road.id, self.id, road.src, road.dst
            );
        }
    }

    fn fill_lanes(&mut self, incoming: Vec<LaneID>, outgoing: Vec<LaneID>) {
        self.outgoing_lanes.extend(outgoing);
        self.incoming_lanes.extend(incoming);
    }

    pub fn update_traffic_control(&self, roads: &Roads, lanes: &mut Lanes) {
        self.light_policy.apply(self, roads, lanes);
    }
}

pub fn make_inter_entity<'a>(
    inter: &Intersection,
    inter_pos: Vector2<f32>,
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
                color: Color { a: 0.5, ..BLUE },
                filled: true,
                ..CircleRender::default()
            },
            2,
        ))
        .with(Transform::new(inter_pos))
        .with(Movable)
        .with(Selectable::default())
        .build()
}
