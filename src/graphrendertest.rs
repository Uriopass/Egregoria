use crate::engine::components::{
    CircleRender, LineToRender, MeshRender, MeshRenderBuilder, Movable, Position,
};
use crate::graphs::graph::{Graph, NodeID};
use cgmath::Vector2;
use ggez::graphics::{Color, BLACK, WHITE};
use specs::storage::BTreeStorage;
use specs::{Builder, Component, Entity, World, WorldExt};
use std::collections::HashMap;

#[derive(Component)]
#[storage(BTreeStorage)]
pub struct Node {
    id: NodeID,
}

pub fn setup(world: &mut World) {
    let mut g = Graph::new();

    let a = g.add_node(Vector2::<f32>::new(0., 0.));
    let b = g.add_node(Vector2::<f32>::new(100., 100.));
    let c = g.add_node(Vector2::<f32>::new(-100., 100.));

    g.add_neigh(a, b, 1.);
    g.add_neigh(a, c, 1.);
    g.add_neigh(c, b, 1.);

    let mut e_map: HashMap<NodeID, Entity> = HashMap::new();
    for n in g.ids() {
        e_map.insert(
            *n,
            world
                .create_entity()
                .with(Node { id: *n })
                .with(Position(g.nodes[n]))
                .with(Movable)
                .build(),
        );
    }

    let mut meshrenders = world.write_component::<MeshRender>();

    for n in g.ids() {
        let e = e_map[n];

        let mut meshb = MeshRenderBuilder::new().add(CircleRender {
            radius: 10.0,
            color: Color { g: 1.0, ..BLACK },
            filled: true,
        });

        for nei in g.get_neighs(*n) {
            let e_nei = e_map[&nei.to];
            meshb = meshb.add(LineToRender {
                color: WHITE,
                to: e_nei,
            });
        }

        meshrenders
            .insert(e, meshb.build())
            .expect("Error inserting mesh for graph");
    }
}
