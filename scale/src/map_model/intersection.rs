use crate::gui::InspectDragf;
use crate::interaction::{Movable, Selectable};
use crate::map_model::{
    Intersections, LaneID, Lanes, Road, RoadID, Roads, TrafficControl, TrafficLightSchedule, Turn,
    TurnID, TurnPolicy,
};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::{Color, BLUE};
use cgmath::{InnerSpace, Vector2};
use imgui_inspect_derive::*;
use ordered_float::OrderedFloat;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use specs::storage::BTreeStorage;
use specs::{Builder, Component, Entities, Entity, LazyUpdate};
use std::collections::BTreeMap;

new_key_type! {
    pub struct IntersectionID;
}

#[derive(Component, Clone, Serialize, Deserialize, Inspect)]
#[storage(BTreeStorage)]
pub struct IntersectionComponent {
    #[inspect(skip = true)]
    pub id: IntersectionID,
    #[inspect(proxy_type = "InspectDragf")]
    pub radius: f32,
    pub policy: TurnPolicy,
}

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    pub pos: Vector2<f32>,

    pub turns: BTreeMap<TurnID, Turn>,
    pub policy: TurnPolicy,

    pub incoming_lanes: Vec<LaneID>,
    pub outgoing_lanes: Vec<LaneID>,

    pub roads: Vec<RoadID>,
    pub interface_radius: f32,
}

impl Intersection {
    pub fn make(store: &mut Intersections, pos: Vector2<f32>) -> IntersectionID {
        store.insert_with_key(|id| Intersection {
            id,
            pos,
            turns: BTreeMap::new(),
            policy: TurnPolicy::default(),
            incoming_lanes: vec![],
            outgoing_lanes: vec![],
            roads: vec![],
            interface_radius: 20.0,
        })
    }

    pub fn clean(&mut self, lanes: &Lanes, roads: &Roads) {
        self.incoming_lanes.retain(|x| lanes.contains_key(*x));
        self.outgoing_lanes.retain(|x| lanes.contains_key(*x));

        self.roads.retain(|x| roads.contains_key(*x));

        self.gen_turns(lanes, roads);
    }

    pub fn gen_turns(&mut self, lanes: &Lanes, roads: &Roads) {
        let turns = self.policy.generate_turns(self, lanes, roads);

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

    fn pseudo_angle(v: Vector2<f32>) -> f32 {
        debug_assert!((v.magnitude2() - 1.0).abs() <= 1e-5);
        let dx = v.x;
        let dy = v.y;
        let p = dx / (dx.abs() + dy.abs());

        if dy < 0.0 {
            p - 1.0
        } else {
            1.0 - p
        }
    }

    pub fn update_traffic_control(&self, roads: &Roads, lanes: &mut Lanes) {
        let mut in_road_lanes: Vec<&Vec<LaneID>> = self
            .roads
            .iter()
            .map(|x| roads[*x].incoming_lanes_from(self.id))
            .filter(|v| !v.is_empty())
            .collect();

        if in_road_lanes.len() <= 2 {
            for incoming_lanes in in_road_lanes {
                for lane in incoming_lanes {
                    lanes[*lane].control = TrafficControl::Always;
                }
            }
            return;
        }

        in_road_lanes.sort_by_key(|x| {
            OrderedFloat(Self::pseudo_angle(
                roads[lanes[*x.first().unwrap()].parent].dir_from(self),
            ))
        });

        let cycle_size = 10;
        let orange_length = 5;
        let offset = self.id.0.as_ffi() as u32;
        let offset: usize =
            rand::rngs::SmallRng::seed_from_u64(offset as u64).gen_range(0, cycle_size);

        for (i, incoming_lanes) in in_road_lanes.into_iter().enumerate() {
            let light = TrafficControl::Periodic(TrafficLightSchedule::from_basic(
                cycle_size,
                orange_length,
                cycle_size + orange_length,
                if i % 2 == 0 {
                    cycle_size + orange_length + offset
                } else {
                    offset
                },
            ));

            for lane in incoming_lanes {
                lanes[*lane].control = light;
            }
        }
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
            policy: inter.policy,
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
        .with(Selectable)
        .build()
}
