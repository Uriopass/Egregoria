use crate::cars::car_data::{make_car_entity, CarComponent};
use crate::graphs::graph::NodeID;
use crate::map::{make_inter_entity, RGSData, RoadGraph};
use crate::physics::physics_components::{Kinematics, Transform};
use cgmath::InnerSpace;
use cgmath::Vector2;
use imgui_inspect_derive::*;
use rand::random;
use specs::error::NoError;
use specs::saveload::SimpleMarker;
use specs::storage::BTreeStorage;
use specs::{Component, World, WorldExt, WriteStorage};
use std::fs::File;

pub mod car_data;
pub mod car_system;

#[allow(dead_code)]
#[derive(Component, Inspect, Clone)]
#[storage(BTreeStorage)]
pub struct IntersectionComponent {
    #[inspect(skip)]
    pub id: NodeID,
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

type CarLoadComponents<'a> = (
    WriteStorage<'a, Transform>,
    WriteStorage<'a, Kinematics>,
    WriteStorage<'a, CarComponent>,
);

#[derive(Clone, Copy)]
pub struct CarMarker;

#[rustfmt::skip]
pub fn setup(world: &mut World) {
    let rg = RoadGraph::from_file("graph").unwrap_or(RoadGraph::empty());

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
    
    let entities = world.entities();
    let file = File::create("cars.json").unwrap();
    let mut ser = serde_json::Serializer::new(file);

    specs::saveload::SerializeComponents::<NoError, SimpleMarker<CarMarker>>::serialize(&world.system_data::<CarLoadComponents>(), &entities, &world.read_component::<SimpleMarker<CarMarker>>(), &mut ser).unwrap();
}
