use cgmath::Vector2;
use specs::{Component, Entity, VecStorage, World, WorldExt};

mod kinematics;
pub mod systems;
mod transform;

use crate::geometry::gridstore::{GridStore, GridStoreHandle};
pub use kinematics::*;
pub use transform::*;

#[derive(Clone, Copy)]
pub struct PhysicsObject {
    pub dir: Vector2<f32>,
    pub speed: f32,
    // e: Entity, // Maybe I'll need this someday
}

pub type PhysicsWorld = GridStore<PhysicsObject>;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub GridStoreHandle);

pub fn add_to_coworld(world: &mut World, e: Entity) {
    let trans = world
        .read_component::<transform::Transform>()
        .get(e)
        .unwrap()
        .clone();
    let coworld = world.get_mut::<PhysicsWorld>().unwrap();
    let h = coworld.insert(
        trans.position(),
        PhysicsObject {
            dir: trans.direction(),
            speed: 0.0,
        },
    );

    let mut collider_comp = world.write_component::<Collider>();
    collider_comp.insert(e, Collider(h)).unwrap();
}
