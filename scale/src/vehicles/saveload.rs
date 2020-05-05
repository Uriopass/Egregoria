use crate::physics::Transform;
use crate::vehicles::make_vehicle_entity;
use crate::vehicles::VehicleComponent;
use specs::{Join, World, WorldExt};
use std::fs::File;

const VEHICLE_FILENAME: &str = "world/vehicle";

pub fn save(world: &mut World) {
    let _ = std::fs::create_dir("world");

    let path = VEHICLE_FILENAME.to_string() + ".bc";
    let file = File::create(path).unwrap();

    let storages = (
        &world.read_component::<Transform>(),
        &world.read_component::<VehicleComponent>(),
    );

    let comps: Vec<_> = storages.join().map(|(trans, car)| (trans, car)).collect();

    bincode::serialize_into(file, &comps).unwrap();
}

pub fn load(world: &mut World) {
    let file = File::open(VEHICLE_FILENAME.to_string() + ".bc");
    if let Err(e) = file {
        println!("error while trying to load entities: {}", e);
        return;
    }

    let des = bincode::deserialize_from(file.unwrap());

    let comps: Vec<(Transform, VehicleComponent)> = des.unwrap_or_default();

    for (trans, car) in comps {
        make_vehicle_entity(world, trans, car);
    }
}
