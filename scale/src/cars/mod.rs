use crate::cars::data::{make_car_entity, CarComponent};
use crate::map::{make_inter_entity, RoadGraph};
use crate::physics::physics_components::Transform;

use cgmath::InnerSpace;
use cgmath::Vector2;
use rand::random;
use specs::{Join, LazyUpdate, World, WorldExt};
use std::fs::File;

pub mod data;
pub mod systems;

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

    let car = CarComponent::new((node_pos - pos).normalize());

    make_car_entity(world, Transform::new(pos), car);
}

const CAR_FILENAME: &str = "world/cars";
const GRAPH_FILENAME: &str = "world/graph";

pub fn save(world: &mut World) {
    world.read_resource::<RoadGraph>().save(GRAPH_FILENAME);

    let file = File::create(CAR_FILENAME.to_string() + ".json").unwrap();

    let car_trans: Vec<(Transform, CarComponent)> = (
        &world.read_component::<Transform>(),
        &world.read_component::<CarComponent>(),
    )
        .join()
        .map(|(trans, car)| (trans.clone(), car.clone()))
        .collect();

    serde_json::to_writer_pretty(file, &car_trans).unwrap();
}

pub fn load(world: &mut World) {
    {
        let rg = RoadGraph::from_file(GRAPH_FILENAME).unwrap_or_else(RoadGraph::empty);
        for (inter_id, inter) in rg.intersections() {
            make_inter_entity(
                *inter_id,
                inter.pos,
                &world.read_resource::<LazyUpdate>(),
                &world.entities(),
            );
        }
        world.insert(rg);
    }

    let file = File::open(CAR_FILENAME.to_string() + ".json");
    if let Err(e) = file {
        println!("error while trying to load entities: {}", e);
        return;
    }

    let des = serde_json::from_reader(file.unwrap());

    let ok: Vec<(Transform, CarComponent)> = des.unwrap_or_default();

    for (trans, car) in ok {
        make_car_entity(world, trans, car);
    }
}

#[rustfmt::skip]
pub fn setup(world: &mut World) {
    load(world);
}
