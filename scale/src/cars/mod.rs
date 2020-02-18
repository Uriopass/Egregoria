use crate::cars::data::{make_car_entity, CarComponent, CarObjective};
use crate::map_model::Map;
use crate::physics::Transform;
use cgmath::InnerSpace;
use specs::{Join, World, WorldExt};
use std::fs::File;

pub mod data;
pub mod systems;

const CAR_FILENAME: &str = "world/cars";

pub fn spawn_new_car(world: &mut World) {
    let mut pos = [0.0, 0.0].into();
    let mut dir = [1.0, 0.0].into();
    let mut obj = CarObjective::None;

    {
        let navmesh = &world.read_resource::<Map>().navmesh;
        let l = navmesh.len();
        if l > 0 {
            loop {
                let r = (rand::random::<f32>() * l as f32) as usize;
                let (nav_id, nav) = navmesh.into_iter().nth(r).unwrap();
                let back = navmesh.get_backward_neighs(*nav_id);
                let l2 = back.len();
                if l2 == 0 {
                    continue;
                }
                let r2 = (rand::random::<f32>() * l2 as f32) as usize;

                let backnode = &navmesh[back.into_iter().nth(r2).unwrap().to];

                let diff = nav.pos - backnode.pos;

                if diff.magnitude() < 10.0 {
                    continue;
                }

                pos = backnode.pos + rand::random::<f32>() * diff;
                dir = diff.normalize();
                obj = CarObjective::Temporary(*nav_id);
                break;
            }
        }
    }

    let car = CarComponent::new(dir, obj);

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
