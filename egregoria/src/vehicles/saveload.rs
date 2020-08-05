use crate::map_interaction::Itinerary;
use crate::physics::Transform;
use crate::vehicles::VehicleComponent;
use specs::{Join, World, WorldExt};
use std::fs::File;

const VEHICLE_FILENAME: &str = "world/vehicle";

pub fn save(world: &mut World) {
    let storages = (
        &world.read_component::<Transform>(),
        &world.read_component::<VehicleComponent>(),
        &world.read_component::<Itinerary>(),
    );

    let comps: Vec<_> = storages
        .join()
        .map(|(trans, car, it)| (trans, car, it))
        .collect();

    let _ = crate::saveload::save(&comps, "vehicles");
}

pub fn load(_world: &mut World) {
    let file = File::open(VEHICLE_FILENAME.to_string() + ".bc");
    if let Err(e) = file {
        error!("failed loading entities: {}", e);
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
