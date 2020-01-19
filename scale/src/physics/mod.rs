use crate::interaction::Movable;
use crate::physics::physics_components::{Collider, Transform};
use crate::rendering::meshrender_component::{LineRender, MeshRender};
use crate::rendering::GREEN;
use cgmath::Vector2;
use ncollide2d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide2d::shape::{Segment, Shape, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use specs::{Builder, Entity, World, WorldExt};

pub mod physics_components;
pub mod physics_system;

pub type PhysicsWorld = CollisionWorld<f32, Entity>;

pub fn add_shape<T>(world: &mut World, e: Entity, shape: T)
where
    T: Shape<f32>,
{
    let pos = world
        .read_component::<Transform>()
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

pub fn add_static_segment(world: &mut World, start: Vector2<f32>, offset: Vector2<f32>) {
    let e = world
        .create_entity()
        .with(Transform::new(start))
        .with(MeshRender::simple(LineRender {
            offset,
            color: GREEN,
            thickness: 0.2,
        }))
        .with(Movable)
        .build();

    add_shape(
        world,
        e,
        Segment::new(
            nalgebra::Point2::new(0.0, 0.0),
            nalgebra::Point2::new(offset.x, offset.y),
        ),
    );
}
