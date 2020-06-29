use crate::map_interaction::Itinerary;
use crate::physics::Transform;
use crate::vehicles::VehicleComponent;
use specs::{Join, World, WorldExt};
use std::fs::File;

const VEHICLE_FILENAME: &str = "world/vehicle";

pub fn save(world: &mut World) {
    let _ = std::fs::create_dir("world");

    let path = VEHICLE_FILENAME.to_string() + ".bc";
    let file = unwrap_or!(File::create(path).ok(), {
        println!("Couldn't create vehicle file");
        return;
    });

    let storages = (
        &world.read_component::<Transform>(),
        &world.read_component::<VehicleComponent>(),
        &world.read_component::<Itinerary>(),
    );

    let comps: Vec<_> = storages
        .join()
        .map(|(trans, car, it)| (trans, car, it))
        .collect();

    let _ = bincode::serialize_into(file, &comps);
}

pub fn load(_world: &mut World) {
    let file = File::open(VEHICLE_FILENAME.to_string() + ".bc");
    if let Err(e) = file {
        println!("error while trying to load entities: {}", e);
        return;
    }

    /*
    // FIXME: load parked cars and shit
    let des = bincode::deserialize_from(file.unwrap());

    let comps: Vec<(Transform, VehicleComponent, Itinerary)> = des.unwrap_or_default();

    for (trans, car, it) in comps {
        make_vehicle_entity(world, trans, car, it);
    }
     */
}
