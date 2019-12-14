use crate::cars::car::make_car_entity;
use crate::cars::car_graph::RoadGraph;
use crate::graphs::graph::NodeID;
use cgmath::Vector2;
use specs::storage::BTreeStorage;
use specs::{Builder, Component, Entity, World, WorldExt};

pub mod car;
pub mod car_graph;
pub mod car_system;

#[derive(Component)]
#[storage(BTreeStorage)]
pub struct RoadNodeComponent {
    id: NodeID,
}

pub fn setup(world: &mut World) {
    let g = RoadGraph::new();
    g.add_to_world(world);
    world.insert(g);

    for i in 0..10 {
        make_car_entity(
            world,
            Vector2::<f32>::new(rand::random(), rand::random()) * 1000.,
            Vector2::new(500., 500.),
        );
    }
}
