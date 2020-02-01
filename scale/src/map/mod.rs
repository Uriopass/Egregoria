use crate::interaction::{Movable, Selectable};
use crate::map::traffic_lights::TrafficLight;
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::RED;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use specs::storage::BTreeStorage;
use specs::{Builder, Component, Entities, Entity, LazyUpdate, World, WorldExt};
use std::collections::HashMap;

mod road_graph;
mod road_graph_synchronize;
mod traffic_lights;

pub use road_graph::RoadGraph;
pub use road_graph_synchronize::*;
pub use traffic_lights::*;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadNodeID(pub usize);
impl From<usize> for RoadNodeID {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntersectionID(pub usize);
impl From<usize> for IntersectionID {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RoadNode {
    pub pos: Vector2<f32>,
    pub light: TrafficLight,
}

impl RoadNode {
    pub fn new(pos: Vector2<f32>) -> Self {
        RoadNode {
            pos,
            light: TrafficLight::Always,
        }
    }
}

const GRAPH_FILENAME: &str = "world/graph";

pub fn save(world: &mut World) {
    world.read_resource::<RoadGraph>().save(GRAPH_FILENAME);
}

pub fn load(world: &mut World) {
    let rg = RoadGraph::from_file(GRAPH_FILENAME).unwrap_or_else(RoadGraph::empty);
    for (inter_id, inter) in rg.intersections() {
        make_inter_entity(
            *inter_id,
            inter.pos,
            &world.read_resource::<LazyUpdate>(),
            &world.entities(),
        );
    }
    world.insert(rg);
}

pub fn setup(world: &mut World) {
    load(world);
}

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub pos: Vector2<f32>,
    pub out_nodes: HashMap<IntersectionID, RoadNodeID>,
    pub in_nodes: HashMap<IntersectionID, RoadNodeID>,
}

impl Intersection {
    pub fn new(pos: Vector2<f32>) -> Self {
        Intersection {
            pos,
            out_nodes: HashMap::new(),
            in_nodes: HashMap::new(),
        }
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
