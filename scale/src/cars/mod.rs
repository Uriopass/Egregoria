use crate::cars::data::{make_car_entity, CarComponent};
use crate::map::RoadGraph;
use crate::physics::Transform;
use cgmath::InnerSpace;
use cgmath::Vector2;
use rand::random;
use specs::{Join, World, WorldExt};
use std::fs::File;

pub mod data;
pub mod systems;

const CAR_FILENAME: &str = "world/cars";

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

pub fn save(world: &mut World) {
    let file = File::create(CAR_FILENAME.to_string() + ".bc").unwrap();

    let car_trans: Vec<(Transform, CarComponent)> = (
        &world.read_component::<Transform>(),
        &world.read_component::<CarComponent>(),
    )
        .join()
        .map(|(trans, car)| (trans.clone(), car.clone()))
        .collect();

    bincode::serialize_into(file, &car_trans).unwrap();
}

pub fn load(world: &mut World) {
    let file = File::open(CAR_FILENAME.to_string() + ".bc");
    if let Err(e) = file {
        println!("error while trying to load entities: {}", e);
        return;
    }

    let des = bincode::deserialize_from(file.unwrap());

    let ok: Vec<(Transform, CarComponent)> = des.unwrap_or_default();

    for (trans, car) in ok {
        make_car_entity(world, trans, car);
    }
}

pub fn setup(world: &mut World) {
    load(world);
}
