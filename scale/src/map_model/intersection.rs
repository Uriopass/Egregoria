use crate::interaction::{Movable, Selectable};
use crate::map_model::navmesh::NavNodeID;
use crate::map_model::TrafficLight::Always;
use crate::map_model::{
    LaneID, Map, NavMesh, NavNode, Road, RoadID, TrafficLight, TrafficLightSchedule, Turn,
};
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::RED;
use cgmath::{InnerSpace, Vector2};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use slab::Slab;
use specs::storage::BTreeStorage;
use specs::{Builder, Component, Entities, Entity, LazyUpdate};
use std::collections::HashMap;
use std::ops::Sub;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntersectionID(pub usize);
impl From<usize> for IntersectionID {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    id: IntersectionID,
    pub pos: Vector2<f32>,
    pub turns: Vec<Turn>,

    pub incoming_lanes: Vec<LaneID>,
    pub outgoing_lanes: Vec<LaneID>,

    pub roads: Vec<RoadID>,

    pub out_nodes: HashMap<LaneID, NavNodeID>,
    pub in_nodes: HashMap<LaneID, NavNodeID>,
}

impl Intersection {
    pub fn make(store: &mut Slab<Intersection>, pos: Vector2<f32>) -> &mut Intersection {
        let entry = store.vacant_entry();
        let id = IntersectionID(entry.key());
        entry.insert(Intersection {
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

    pub fn gen_interface_navmesh(&mut self, map: &mut Map) {
        for lane_id in &self.incoming_lanes {
            if self.in_nodes.contains_key(lane_id) {
                continue;
            }
            let lane = &map.lanes[lane_id.0];
            let road = &map.roads[lane.parent.0];

            let lane_dist = road.idx_unchecked(*lane_id);
            let dir = road.dir_from(self);
            let dir_normal: Vector2<f32> = [-dir.y, dir.x].into();

            let pos = dir * 10.0 + dir_normal * lane_dist as f32;

            let nav_id = map.navmesh.push(NavNode::new(pos));
            self.in_nodes.insert(*lane_id, nav_id);
        }

        for lane_id in &self.outgoing_lanes {
            if self.out_nodes.contains_key(lane_id) {
                continue;
            }
            let lane = &map.lanes[lane_id.0];
            let road = &map.roads[lane.parent.0];

            let lane_dist = road.idx_unchecked(*lane_id);
            let dir = road.dir_from(self);
            let dir_normal: Vector2<f32> = [-dir.y, dir.x].into();

            let pos = dir * 10.0 - dir_normal * lane_dist as f32;

            let nav_id = map.navmesh.push(NavNode::new(pos));
            self.in_nodes.insert(*lane_id, nav_id);
        }
    }

    pub fn id(&self) -> IntersectionID {
        self.id
    }

    pub fn add_road(&mut self, road: &Road) {
        self.roads.push(road.id());
        if road.src == self.id {
            self.fill_lanes(road.lanes_backward.clone(), road.lanes_forward.clone());
        } else if road.dst == self.id {
            self.fill_lanes(road.lanes_forward.clone(), road.lanes_backward.clone());
        } else {
            panic!(
                "Trying to add {:?} to {:?} but it's between {:?} and {:?}",
                road.id(),
                self.id,
                road.src,
                road.dst
            );
        }
    }

    fn fill_lanes(&mut self, incoming: Vec<LaneID>, outgoing: Vec<LaneID>) {
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

        self.outgoing_lanes.extend(outgoing);
        self.incoming_lanes.extend(incoming);
    }

    pub fn add_turn(&mut self, src: LaneID, dst: LaneID) {
        self.turns.push(Turn {
            parent: self.id,
            src,
            dst,
            nodes: vec![],
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
