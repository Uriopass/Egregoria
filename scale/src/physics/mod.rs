use specs::{Component, Entity, VecStorage, World, WorldExt};
pub mod systems;

mod kinematics;
mod transform;

use crate::geometry::gridstore::{GridStore, GridStoreHandle};
pub use kinematics::*;
pub use transform::*;

pub type PhysicsWorld = GridStore<Entity>;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub GridStoreHandle);

pub fn add_to_coworld(world: &mut World, e: Entity) {
    let pos = world
        .read_component::<transform::Transform>()
        .get(e)
        .unwrap()
        .position();
    let coworld = world.get_mut::<PhysicsWorld>().unwrap();
    let h = coworld.insert(pos, e);

    let mut collider_comp = world.write_component::<Collider>();
    collider_comp.insert(e, Collider(h)).unwrap();
}
