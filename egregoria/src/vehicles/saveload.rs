use crate::map_dynamic::Itinerary;
use crate::physics::Transform;
use crate::vehicles::VehicleComponent;
use specs::{Join, World, WorldExt};

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
    /*
    // FIXME: load parked cars and shit
    let des = bincode::deserialize_from(file.unwrap());

    let comps: Vec<(Transform, VehicleComponent, Itinerary)> = des.unwrap_or_default();

    for (trans, car, it) in comps {
        make_vehicle_entity(world, trans, car, it);
    }
     */
}
