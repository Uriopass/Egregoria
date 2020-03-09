use crate::physics::Transform;
use crate::transportation::make_transport_entity;
use crate::transportation::transport_component::TransportComponent;
use specs::{Join, World, WorldExt};
use std::fs::File;

const TRANSPORT_FILENAME: &str = "world/transport";

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
