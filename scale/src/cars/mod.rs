use crate::cars::car_data::make_car_entity;

use crate::graphs::graph::NodeID;
use crate::interaction::{Movable, Selectable};
use crate::physics::physics_components::Transform;
use crate::rendering::meshrender_component::MeshRender;
use cgmath::InnerSpace;
use cgmath::Vector2;
use imgui_inspect_derive::*;
use rand::random;
use map::Intersection;
use map::RoadGraph;
use specs::storage::BTreeStorage;
use specs::{Component, World, WorldExt};

pub mod car_data;
pub mod car_system;
pub mod map;

#[allow(dead_code)]
#[derive(Component, Inspect, Clone)]
#[storage(BTreeStorage)]
pub struct IntersectionComponent {
    #[inspect(skip)]
    id: NodeID,
}

pub fn spawn_new_car(world: &mut World) {
    let node_pos = {
        let rg = world.read_resource::<RoadGraph>();
        let l = rg.nodes().len();
        let r = (rand::random::<f32>() * l as f32) as usize;

        rg.nodes().into_iter().skip(r).next().unwrap().1.pos
    };
    let pos = node_pos
        + Vector2::new(
            10.0 * (random::<f32>() - 0.5),
            10.0 * (random::<f32>() - 0.5),
        );

    make_car_entity(world, pos, (node_pos - pos).normalize());
}

#[rustfmt::skip]
pub fn setup(world: &mut World) {
    let mut rg = RoadGraph::new();
    let center = rg.add_intersection(Intersection::new([0.0, 0.0].into()));
    let a      = rg.add_intersection(Intersection::new([100.0, 0.0].into()));
    let b      = rg.add_intersection(Intersection::new([-100.0, 0.0].into()));
    let c      = rg.add_intersection(Intersection::new([0.0, 100.0].into()));
    let d      = rg.add_intersection(Intersection::new([0.0, -100.0].into()));

    rg.connect(&a, &center);
    rg.connect(&b, &center);
    rg.connect(&c, &center);
    rg.connect(&d, &center);

    rg.connect(&a, &c);
    rg.connect(&c, &b);
    rg.connect(&b, &d);
    rg.connect(&d, &a);

    world.insert(rg);

    for x in &[a, b, c, d, center] {
        world.write_resource::<RoadGraph>().make_entity(
            x,
            &world.entities(),
            &mut world.write_component::<IntersectionComponent>(),
            &mut world.write_component::<MeshRender>(),
            &mut world.write_component::<Transform>(),
            &mut world.write_component::<Movable>(),
            &mut world.write_component::<Selectable>(),
        );
    }

    for _i in 0..10 {
        spawn_new_car(world);
    }
}
