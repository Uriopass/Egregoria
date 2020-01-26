use crate::cars::car_data::make_car_entity;
use crate::cars::map::{make_inter_entity, RGSData};
use crate::graphs::graph::NodeID;
use cgmath::InnerSpace;
use cgmath::Vector2;
use imgui_inspect_derive::*;
use map::RoadGraph;
use rand::random;
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
    let node_pos: Vector2<f32> = {
        let rg = world.read_resource::<RoadGraph>();
        let l = rg.nodes().len();
        if l == 0 {
            [0.0, 0.0].into()
        } else {
            let r = (rand::random::<f32>() * l as f32) as usize;
            rg.nodes().into_iter().nth(r).unwrap().1.pos
        }
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
    let rg = RoadGraph::from_file("graph.bc").unwrap_or(RoadGraph::empty());
    
    world.insert(rg);

    let nudes: Vec<NodeID> = world.read_resource::<RoadGraph>().intersections().into_iter().map(|(a, _)| *a).collect();
    for id in nudes {
        let inter = world.read_resource::<RoadGraph>().intersections()[id].pos;
        
        let mut data = world.system_data::<RGSData>();
        make_inter_entity(
            id,
            inter,
            &mut data
        );
    }

    for _i in 0..10 {
        spawn_new_car(world);
    }
}
