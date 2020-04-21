use crate::interaction::Selectable;
use crate::map_model::{Itinerary, LaneKind, Map, Traversable, TraverseDirection, TraverseKind};
use crate::physics::{
    Collider, CollisionWorld, Kinematics, PhysicsGroup, PhysicsObject, Transform,
};
use crate::rendering::meshrender_component::MeshRender;
use crate::utils::rand_det;
use cgmath::InnerSpace;
use specs::{Builder, Entity, World, WorldExt};

mod data;
mod saveload;
pub mod systems;

use crate::rendering::assets::{AssetID, AssetRender};
pub use data::*;
pub use saveload::*;

pub fn spawn_new_vehicle(world: &mut World) {
    let map = world.read_resource::<Map>();

    if let Some(lane) = map.get_random_lane(LaneKind::Driving) {
        if let [a, b, ..] = lane.points.as_slice() {
            let diff = b - a;

            let mut pos = Transform::new(*a + rand_det::<f32>() * diff);
            pos.set_direction(diff.normalize());

            let mut it = Itinerary::default();
            it.set_simple(
                Traversable::new(TraverseKind::Lane(lane.id), TraverseDirection::Forward),
                &map,
            );
            it.advance(&map);

            drop(map);
            make_vehicle_entity(world, pos, VehicleComponent::new(it, VehicleKind::Car));
        }
    }
}

pub fn make_vehicle_entity(
    world: &mut World,
    trans: Transform,
    vehicle: VehicleComponent,
) -> Entity {
    let mut mr = MeshRender::empty(3);
    vehicle.kind.build_mr(&mut mr);

    let coworld = world.get_mut::<CollisionWorld>().unwrap();
    let h = coworld.insert(
        trans.position(),
        PhysicsObject {
            dir: trans.direction(),
            speed: 0.0,
            radius: vehicle.kind.width(),
            group: PhysicsGroup::Vehicles,
        },
    );

    world
        .create_entity()
        //.with(mr)
        .with(AssetRender {
            id: AssetID::CAR,
            hide: false,
            scale: 4.5,
            tint: get_random_car_color(),
        })
        .with(trans)
        .with(Kinematics::from_mass(1000.0))
        .with(vehicle)
        .with(Collider(h))
        .with(Selectable::default())
        .build()
}

pub fn delete_vehicle_entity(world: &mut World, e: Entity) {
    {
        let handle = world.read_component::<Collider>().get(e).unwrap().0;
        let mut coworld = world.write_resource::<CollisionWorld>();
        coworld.remove(handle);
    }
    world.delete_entity(e).unwrap();
}

pub fn setup(world: &mut World) {
    load(world);
}
