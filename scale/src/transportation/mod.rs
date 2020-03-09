use crate::map_model::{Lane, LaneID, Map, Traversable};
use crate::physics::Transform;
use crate::transportation::data::{make_transport_entity, TransportComponent, TransportObjective};
use cgmath::{vec2, InnerSpace};
use specs::{Join, World, WorldExt};
use std::fs::File;

pub mod data;
pub mod systems;

const TRANSPORT_FILENAME: &str = "world/transport";

pub fn spawn_new_car(world: &mut World) {
    let mut pos = Transform::new(vec2(0.0, 0.0));
    let mut obj = TransportObjective::None;

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
                obj = TransportObjective::Temporary(Traversable::Lane(lane.id));
            }
        }
    }

    let car = TransportComponent::new(obj);

    make_transport_entity(world, pos, car);
}

pub fn save(world: &mut World) {
    let _ = std::fs::create_dir("world");

    let path = TRANSPORT_FILENAME.to_string() + ".bc";
    let file = File::create(path).unwrap();

    let comps: Vec<(Transform, TransportComponent)> = (
        &world.read_component::<Transform>(),
        &world.read_component::<TransportComponent>(),
    )
        .join()
        .map(|(trans, car)| (trans.clone(), car.clone()))
        .collect();

    bincode::serialize_into(file, &comps).unwrap();
}

pub fn load(world: &mut World) {
    let file = File::open(TRANSPORT_FILENAME.to_string() + ".bc");
    if let Err(e) = file {
        println!("error while trying to load entities: {}", e);
        return;
    }

    let des = bincode::deserialize_from(file.unwrap());

    let comps: Vec<(Transform, TransportComponent)> = des.unwrap_or_default();

    for (trans, car) in comps {
        make_transport_entity(world, trans, car);
    }
}

pub fn setup(world: &mut World) {
    load(world);
}
