use ncollide2d::pipeline::{CollisionGroups, CollisionObjectSlabHandle, GeometricQueryType};
use ncollide2d::shape::{Shape, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use specs::{Component, Entity, VecStorage, World, WorldExt};
pub mod systems;

mod kinematics;
mod transform;

pub use kinematics::*;
pub use transform::*;

pub type PhysicsWorld = CollisionWorld<f32, Entity>;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub CollisionObjectSlabHandle);

pub fn add_shape<T>(world: &mut World, e: Entity, shape: T)
where
    T: Shape<f32>,
{
    let pos = world
        .read_component::<transform::Transform>()
        .get(e)
        .unwrap()
        .position();
    let coworld = world.get_mut::<PhysicsWorld>().unwrap();
    let (h, _) = coworld.add(
        nalgebra::Isometry2::new(nalgebra::Vector2::new(pos.x, pos.y), nalgebra::zero()),
        ShapeHandle::new(shape),
        CollisionGroups::new()
            .with_membership(&[1])
            .with_whitelist(&[1]),
        GeometricQueryType::Contacts(0.0, 0.0),
        e,
    );

    let mut collider_comp = world.write_component::<Collider>();
    collider_comp.insert(e, Collider(h)).unwrap();
}
