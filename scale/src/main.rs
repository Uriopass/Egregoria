#![windows_subsystem = "windows"]

use cgmath::{Vector2, Zero};
use ggez::graphics::Color;
use nalgebra as na;
use ncollide2d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide2d::shape::{Segment, Shape, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use specs::{Builder, DispatcherBuilder, Entity, World, WorldExt};

use crate::cars::car_system::CarDecision;
use crate::cars::RoadNodeComponent;
use crate::engine::components::{
    Collider, Kinematics, LineRender, MeshRenderComponent, Movable, Transform,
};
use crate::engine::resources::DeltaTime;
use crate::engine::systems::{KinematicsApply, MovableSystem, PhysicsUpdate};
use crate::humans::HumanUpdate;
use cgmath::num_traits::zero;

mod cars;
mod engine;
mod geometry;
mod graphs;
mod humans;

type PhysicsWorld = CollisionWorld<f32, Entity>;

pub fn add_shape<T>(world: &mut World, e: Entity, shape: T)
where
    T: Shape<f32>,
{
    let pos = world
        .read_component::<Transform>()
        .get(e)
        .unwrap()
        .get_position();
    let coworld = world.get_mut::<PhysicsWorld>().unwrap();
    let (h, _) = coworld.add(
        na::Isometry2::new(na::Vector2::new(pos.x, pos.y), na::zero()),
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
        .with(MeshRenderComponent::simple(LineRender {
            offset,
            color: Color {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        }))
        .with(Movable)
        .build();

    add_shape(
        world,
        e,
        Segment::new(
            na::Point2::new(0.0, 0.0),
            na::Point2::new(offset.x, offset.y),
        ),
    );
}

fn main() {
    let collision_world: PhysicsWorld = CollisionWorld::new(2.0);

    let mut world = World::new();

    world.insert(DeltaTime(0.0));
    world.insert(collision_world);

    world.register::<MeshRenderComponent>();
    world.register::<Collider>();
    world.register::<RoadNodeComponent>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human update", &[])
        .with(CarDecision, "car decision", &[])
        .with(
            MovableSystem::default(),
            "movable",
            &["human update", "car decision"],
        )
        .with(KinematicsApply, "speed apply", &["movable"])
        .with(PhysicsUpdate, "physics", &["speed apply"])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world);
    cars::setup(&mut world);

    engine::start(world, dispatcher);
}
