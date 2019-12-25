use crate::cars::car_data::make_car_entity;
use crate::cars::car_graph::RoadGraph;
use crate::cars::city_generator::{CityGenerator, Intersection};
use crate::graphs::graph::NodeID;
use cgmath::Vector2;
use engine::specs::storage::BTreeStorage;
use engine::specs::{Component, World};

pub mod car_data;
pub mod car_graph;
pub mod car_system;
pub mod city_generator;

#[allow(dead_code)]
#[derive(Component)]
#[storage(BTreeStorage)]
pub struct RoadNodeComponent {
    id: NodeID,
}

pub fn setup(world: &mut World) {
    let mut cb = CityGenerator::new();
    let center = cb.add_intersection(Intersection::new([0.0, 0.0].into()));
    let a = cb.add_intersection(Intersection::new([100.0, 0.0].into()));
    let b = cb.add_intersection(Intersection::new([-100.0, 0.0].into()));
    let c = cb.add_intersection(Intersection::new([0.0, 100.0].into()));
    let d = cb.add_intersection(Intersection::new([0.0, -100.0].into()));

    cb.connect(a, center);
    cb.connect(b, center);
    cb.connect(c, center);
    cb.connect(d, center);

    cb.connect(a, c);
    cb.connect(c, b);
    cb.connect(b, d);
    cb.connect(d, a);

    let g = cb.build();
    g.add_to_world(world);
    world.insert(g);

    for i in 0..30 {
        make_car_entity(
            world,
            200.0 * Vector2::<f32>::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5),
        );
    }
}
