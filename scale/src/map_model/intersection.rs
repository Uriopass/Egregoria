use crate::interaction::{Movable, Selectable};
use crate::map_model::navmesh::NavNodeID;
use crate::map_model::TrafficLight::Always;
use crate::map_model::{
    Intersections, LaneID, Lanes, NavMesh, NavNode, Road, RoadID, Roads, TrafficLight,
    TrafficLightSchedule, Turn,
};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::RED;
use cgmath::{InnerSpace, Vector2};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;
use specs::storage::BTreeStorage;
use specs::{Builder, Component, Entities, Entity, LazyUpdate};
use std::collections::HashMap;
use std::ops::Sub;

new_key_type! {
    pub struct IntersectionID;
}

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    pub pos: Vector2<f32>,
    pub turns: Vec<Turn>,

    pub incoming_lanes: Vec<LaneID>,
    pub outgoing_lanes: Vec<LaneID>,

    pub roads: Vec<RoadID>,

    pub out_nodes: HashMap<LaneID, NavNodeID>,
    pub in_nodes: HashMap<LaneID, NavNodeID>,
}

impl Intersection {
    pub fn make(store: &mut Intersections, pos: Vector2<f32>) -> IntersectionID {
        store.insert_with_key(|id| Intersection {
            id,
            pos,
            turns: vec![],
            incoming_lanes: vec![],
            outgoing_lanes: vec![],
            roads: vec![],
            out_nodes: HashMap::new(),
            in_nodes: HashMap::new(),
        })
    }

    fn get_node_pos(
        &self,
        lane_id: LaneID,
        incoming: bool,
        lanes: &Lanes,
        roads: &Roads,
    ) -> Vector2<f32> {
        let lane = &lanes[lane_id];
        let road = &roads[lane.parent];

        let lane_dist = road.idx_unchecked(lane_id);
        let dir = road.dir_from(self);
        let dir_normal: Vector2<f32> = if incoming {
            [-dir.y, dir.x].into()
        } else {
            [dir.y, -dir.x].into()
        };

        self.pos + dir * 15.0 + dir_normal * lane_dist as f32 * 8.0
    }

    pub fn gen_interface_navmesh(
        &mut self,
        lanes: &mut Lanes,
        roads: &Roads,
        navmesh: &mut NavMesh,
    ) {
        for lane_id in &self.incoming_lanes {
            let pos = self.get_node_pos(*lane_id, true, lanes, roads);
            self.in_nodes
                .entry(*lane_id)
                .and_modify(|x| navmesh.get_mut(*x).unwrap().pos = pos)
                .or_insert_with(|| navmesh.push(NavNode::new(pos)));
        }

        for lane_id in &self.outgoing_lanes {
            let pos = self.get_node_pos(*lane_id, false, lanes, roads);
            self.out_nodes
                .entry(*lane_id)
                .and_modify(|x| navmesh.get_mut(*x).unwrap().pos = pos)
                .or_insert_with(|| navmesh.push(NavNode::new(pos)));
        }
    }

    pub fn gen_turns(&mut self, lanes: &Lanes, navmesh: &mut NavMesh) {
        for turn in &mut self.turns {
            if turn.easing_nodes.is_empty() {
                turn.gen_navmesh(lanes, navmesh);
            } else {
                turn.reposition_nodes(lanes, navmesh);
            }
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
        if self.roads.len() >= 3 {
            for lane_src in self.incoming_lanes.clone() {
                for lane_dst in &outgoing {
                    self.add_turn(lane_src, *lane_dst);
                }
            }
            for lane_dst in self.outgoing_lanes.clone() {
                for lane_src in &incoming {
                    self.add_turn(*lane_src, lane_dst);
                }
            }
        } else if self.roads.len() == 2 {
            for (lane_src, lane_dst) in self.incoming_lanes.clone().into_iter().zip(&outgoing) {
                self.add_turn(lane_src, *lane_dst);
            }
            for (lane_dst, lane_src) in self.outgoing_lanes.clone().into_iter().zip(&incoming) {
                self.add_turn(*lane_src, lane_dst);
            }
        }

        self.outgoing_lanes.extend(outgoing);
        self.incoming_lanes.extend(incoming);
    }

    pub fn add_turn(&mut self, src: LaneID, dst: LaneID) {
        self.turns.push(Turn {
            parent: self.id,
            src,
            dst,
            easing_nodes: vec![],
        });
    }

    fn pseudo_angle(v: Vector2<f32>) -> f32 {
        debug_assert!(v.magnitude2().sub(1.0).abs() <= 1e-5);
        let dx = v.x;
        let dy = v.y;
        let p = dx / (dx.abs() + dy.abs());

        if dy < 0.0 {
            p - 1.0
        } else {
            1.0 - p
        }
    }

    pub fn update_traffic_lights(&mut self, mesh: &mut NavMesh) {
        if self.in_nodes.len() <= 2 {
            for (_, id) in self.in_nodes.iter() {
                mesh[id].light = Always;
            }
            return;
        }
        let mut in_nodes: Vec<&NavNodeID> = self.in_nodes.values().collect();
        in_nodes.sort_by_key(|x| {
            OrderedFloat(Self::pseudo_angle((mesh[*x].pos - self.pos).normalize()))
        });

        let cycle_size = 10;
        let orange_length = 5;
        for (i, id) in in_nodes.into_iter().enumerate() {
            mesh[id].light = TrafficLight::Periodic(TrafficLightSchedule::from_basic(
                cycle_size,
                orange_length,
                cycle_size + orange_length,
                if i % 2 == 0 {
                    cycle_size + orange_length
                } else {
                    0
                },
            ));
        }
    }

    pub fn calculate_nodes_positions(&mut self, _navmesh: &mut NavMesh) {
        let inter = self;
        let _center = inter.pos;
        /*
        for (to, node_id) in &inter.out_nodes {
            let inter2 = &self.intersections[to];
            let center2 = inter2.pos;

            let diff = center2 - center;
            let inter_length = diff.magnitude().max(1e-8);
            let dir = (center2 - center) / inter_length;

            let inter_length = (inter_length / 2.0).min(25.0);

            let nor: Vector2<f32> = Vector2::new(-dir.y, dir.x);

            let rn =navmesh .get_mut(*node_id).unwrap();
            rn.pos = center + dir * inter_length - nor * 4.0;

            let rn2 =navmesh .get_mut(inter2.in_nodes[&i]).unwrap();
            rn2.pos = center2 - dir * inter_length - nor * 4.0;
        }

        for (to, node_id) in &inter.in_nodes {
            let inter2 = &self.intersections[to];
            let center2 = inter2.pos;

            let diff = center2 - center;
            let inter_length = diff.magnitude();
            let dir = (center2 - center) / inter_length;

            let inter_length = (inter_length / 2.0).min(25.0);

            let nor: Vector2<f32> = Vector2::new(-dir.y, dir.x);

            let rn = self.nodes.get_mut(*node_id).unwrap();
            rn.pos = center + dir * inter_length + nor * 4.0;

            let rn2 = self.nodes.get_mut(inter2.out_nodes[&i]).unwrap();
            rn2.pos = center2 - dir * inter_length + nor * 4.0;
        }
            */
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
#[storage(BTreeStorage)]
pub struct IntersectionComponent {
    pub id: IntersectionID,
}
empty_inspect_impl!(IntersectionComponent);

pub fn make_inter_entity<'a>(
    inter_id: IntersectionID,
    inter_pos: Vector2<f32>,
    lazy: &LazyUpdate,
    entities: &Entities<'a>,
) -> Entity {
    lazy.create_entity(entities)
        .with(IntersectionComponent { id: inter_id })
        .with(MeshRender::simple(
            CircleRender {
                radius: 2.0,
                color: RED,
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
