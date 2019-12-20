use crate::cars::car::make_car_entity;
use crate::cars::car_graph::RoadGraph;
use crate::graphs::graph::NodeID;
use cgmath::Vector2;
use specs::storage::BTreeStorage;
use specs::{Component, World};

pub mod car;
pub mod car_graph;
pub mod car_system;

#[allow(dead_code)]
#[derive(Component)]
#[storage(BTreeStorage)]
pub struct RoadNodeComponent {
    id: NodeID,
}

pub fn setup(world: &mut World) {
    let g = RoadGraph::new();
    g.add_to_world(world);
    world.insert(g);

    for i in 0..1 {
        let y = ((i % 3) as f32) * 5.0;
        make_car_entity(
            world,
            Vector2::<f32>::new(i as f32, y) * 2.0,
            Vector2::new(1000.0, y * 2.0),
        );
    }
}
