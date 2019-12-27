use crate::cars::RoadNodeComponent;
use crate::graphs::graph::{Graph, NodeID};
use cgmath::Vector2;
use engine::components::{CircleRender, LineToRender, MeshRenderComponent, Movable, Transform};
use engine::specs::{
    Builder, Entity, Read, ReadStorage, System, SystemData, World, WorldExt, Write,
};

use cgmath::MetricSpace;
use engine::specs::shred::PanicHandler;
use engine::specs::shrev::{EventChannel, ReaderId};
use engine::systems::MovedEvent;
use engine::{GREEN, WHITE};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct RoadNode {
    pub pos: Vector2<f32>,
}

impl RoadNode {
    pub fn new(pos: Vector2<f32>) -> Self {
        RoadNode { pos }
    }
}

pub struct RoadGraph(pub Graph<RoadNode>);
pub struct RoadGraphSynchronize {
    reader: ReaderId<MovedEvent>,
}

impl RoadGraphSynchronize {
    pub fn new(world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(world);
        let reader = world
            .fetch_mut::<EventChannel<MovedEvent>>()
            .register_reader();
        Self { reader }
    }
}

impl<'a> System<'a> for RoadGraphSynchronize {
    type SystemData = (
        Write<'a, RoadGraph, PanicHandler>,
        Read<'a, EventChannel<MovedEvent>>,
        ReadStorage<'a, RoadNodeComponent>,
    );

    fn run(&mut self, (mut road_graph, moved, roadnodecomponents): Self::SystemData) {
        for event in moved.read(&mut self.reader) {
            let rnc = roadnodecomponents.get(event.entity).unwrap();
            road_graph
                .0
                .nodes
                .entry(rnc.id)
                .and_modify(|x| x.pos = event.new_pos);
        }
    }
}

impl RoadGraph {
    pub fn closest_node(&self, pos: Vector2<f32>) -> NodeID {
        let mut id: NodeID = *self.0.ids().next().unwrap();
        let mut min_dist = self.0.nodes.get(&id).unwrap().pos.distance2(pos);

        for (key, value) in &self.0.nodes {
            let dist = pos.distance2(value.pos);
            if dist < min_dist {
                id = *key;
                min_dist = dist;
            }
        }
        id
    }

    pub fn add_to_world(&self, world: &mut World) {
        let g = &self.0;
        let mut e_map: HashMap<NodeID, Entity> = HashMap::new();
        for n in g.ids() {
            e_map.insert(
                *n,
                world
                    .create_entity()
                    .with(RoadNodeComponent { id: *n })
                    .with(Transform::new(g.nodes[n].pos))
                    .with(Movable)
                    .build(),
            );
        }

        let mut meshrenders = world.write_component::<MeshRenderComponent>();

        for n in g.ids() {
            let e = e_map[n];

            let mut meshb = MeshRenderComponent::from(CircleRender {
                radius: 1.0,
                color: GREEN,
                filled: true,
                ..Default::default()
            });

            for nei in g.get_neighs(*n) {
                let e_nei = e_map[&nei.to];
                meshb.add(LineToRender {
                    color: WHITE,
                    to: e_nei,
                });
            }

            meshrenders
                .insert(e, meshb)
                .expect("Error inserting mesh for graph");
        }
    }
}
