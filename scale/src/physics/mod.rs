use cgmath::Vector2;
use specs::{Component, Entity, VecStorage, World, WorldExt};

mod kinematics;
pub mod systems;
mod transform;

use crate::geometry::gridstore::{GridStore, GridStoreHandle};
use crate::vehicles::VehicleComponent;
pub use kinematics::*;
pub use transform::*;

#[derive(Clone, Copy)]
pub struct PhysicsObject {
    pub dir: Vector2<f32>,
    pub speed: f32,
    pub radius: f32,
}

pub type CollisionWorld = GridStore<PhysicsObject>;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub GridStoreHandle);

pub fn add_vehicle_to_coworld(world: &mut World, e: Entity) {
    let trans = world
        .read_component::<transform::Transform>()
        .get(e)
        .unwrap()
        .clone();
    let vehicle = world
        .read_component::<VehicleComponent>()
        .get(e)
        .unwrap()
        .clone();

    let coworld = world.get_mut::<CollisionWorld>().unwrap();
    let h = coworld.insert(
        trans.position(),
        PhysicsObject {
            dir: trans.direction(),
            speed: 0.0,
            radius: vehicle.kind.width(),
        },
    );

    let mut collider_comp = world.write_component::<Collider>();
    collider_comp.insert(e, Collider(h)).unwrap();
}
