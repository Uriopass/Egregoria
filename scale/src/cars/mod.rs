use crate::cars::data::{make_car_entity, CarComponent, CarObjective};
use crate::map_model::{Lane, LaneID, Map, Traversable};
use crate::physics::Transform;
use cgmath::{vec2, InnerSpace};
use specs::{Join, World, WorldExt};
use std::fs::File;

pub mod data;
pub mod systems;

const CAR_FILENAME: &str = "world/cars";

pub fn spawn_new_car(world: &mut World) {
    let mut pos = Transform::new(vec2(0.0, 0.0));
    let mut obj = CarObjective::None;

    {
        let map = world.read_resource::<Map>();
        let roads = map.roads();
        let l = roads.len();
        if l > 0 {
            let r = (rand::random::<f32>() * l as f32) as usize;

            let (_, road) = roads.into_iter().nth(r).unwrap();
            let lanes = road
                .lanes_forward
                .iter()
                .chain(road.lanes_backward.iter())
                .collect::<Vec<&LaneID>>();

            if !lanes.is_empty() {
                let r = (rand::random::<f32>() * lanes.len() as f32) as usize;

                let lane: &Lane = &map.lanes()[*lanes[r]];

                let a = lane.points.first().unwrap();
                let b = lane.points.last().unwrap();

                let diff = b - a;
                pos.set_position(a + rand::random::<f32>() * diff);
                pos.set_direction(diff.normalize());
                obj = CarObjective::Temporary(Traversable::Lane(lane.id));
            }
        }
    }

    let car = CarComponent::new(obj);

    make_car_entity(world, pos, car);
}

pub fn save(world: &mut World) {
    let _ = std::fs::create_dir("world");

    let path = CAR_FILENAME.to_string() + ".bc";
    let file = File::create(path).unwrap();

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
