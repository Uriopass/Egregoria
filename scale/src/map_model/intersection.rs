use crate::interaction::{Movable, Selectable};
use crate::map_model::RoadNodeID;
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use crate::rendering::RED;
use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use specs::storage::BTreeStorage;
use specs::{Builder, Component, Entities, Entity, LazyUpdate};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialOrd, Ord, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntersectionID(pub usize);
impl From<usize> for IntersectionID {
    fn from(x: usize) -> Self {
        Self(x)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Intersection {
    pub id: IntersectionID,
    pub pos: Vector2<f32>,
    pub out_nodes: HashMap<IntersectionID, RoadNodeID>,
    pub in_nodes: HashMap<IntersectionID, RoadNodeID>,
}

impl Intersection {
    pub fn new(pos: Vector2<f32>) -> Self {
        Intersection {
            id: 0,
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
