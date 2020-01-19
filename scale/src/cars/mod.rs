use crate::cars::car_data::make_car_entity;

use crate::graphs::graph::NodeID;
use crate::interaction::{Movable, Selectable};
use crate::physics::physics_components::Transform;
use cgmath::Vector2;
use imgui_inspect_derive::*;
use roads::road_graph::RoadGraph;
use roads::Intersection;
use specs::storage::BTreeStorage;
use specs::{Component, Entities, World, WorldExt};

pub mod car_data;
pub mod car_system;
pub mod roads;

#[allow(dead_code)]
#[derive(Component, Inspect, Clone)]
#[storage(BTreeStorage)]
pub struct RoadNodeComponent {
    #[inspect(skip)]
    id: NodeID,
}

#[allow(dead_code)]
#[derive(Component, Inspect, Clone)]
#[storage(BTreeStorage)]
pub struct IntersectionComponent {
    #[inspect(skip)]
    id: NodeID,
}

#[rustfmt::skip]
pub fn setup(world: &mut World) {
    let mut rg = RoadGraph::new();
    let center = rg.add_intersection(Intersection::new([0.0, 0.0].into()));
    let a      = rg.add_intersection(Intersection::new([100.0, 0.0].into()));
    let b      = rg.add_intersection(Intersection::new([-100.0, 0.0].into()));
    let c      = rg.add_intersection(Intersection::new([0.0, 100.0].into()));
    let d      = rg.add_intersection(Intersection::new([0.0, -100.0].into()));

    rg.connect(a, center);
    rg.connect(b, center);
    rg.connect(c, center);
    rg.connect(d, center);

    rg.connect(a, c);
    rg.connect(c, b);
    rg.connect(b, d);
    rg.connect(d, a);

    rg.build_nodes();
    world.insert(rg);
    
    world.write_resource::<RoadGraph>().populate_entities(
        &world.entities(),
        &mut world.write_component::<RoadNodeComponent>(),
        &mut world.write_component::<IntersectionComponent>(),
        &mut world.write_component::<Transform>(),
        &mut world.write_component::<Movable>(),
        &mut world.write_component::<Selectable>(),
    );

    for _i in 0..10 {
        make_car_entity(
            world,
            200.0 * Vector2::<f32>::new(rand::random::<f32>() - 0.5, rand::random::<f32>() - 0.5),
        );
    }
}
