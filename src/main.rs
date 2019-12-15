#![windows_subsystem = "windows"]

use crate::engine::components::{Collider, LineRender, MeshRenderComponent, Transform};
use crate::engine::resources::DeltaTime;
use crate::engine::systems::{KinematicsApply, MovableSystem, PhysicsUpdate};
use crate::humans::HumanUpdate;
use cgmath::{Vector2, Zero};
use ggez::graphics::Color;
use ncollide2d::shape::{Segment, Shape, ShapeHandle};
use ncollide2d::world::CollisionWorld;

mod cars;
mod engine;
mod geometry;
mod graphs;
mod humans;

use crate::cars::car_system::CarDecision;
use crate::cars::RoadNodeComponent;
use nalgebra as na;
use ncollide2d::pipeline::{CollisionGroups, GeometricQueryType};
use specs::{Builder, DispatcherBuilder, Entity, World, WorldExt};

type PhysicsWorld = CollisionWorld<f32, Entity>;

pub fn add_shape<T>(world: &mut World, e: Entity, pos: Vector2<f32>, shape: T)
where
    T: Shape<f32>,
{
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

pub fn add_static_segment(world: &mut World, start: Vector2<f32>, end: Vector2<f32>) {
    let e = world
        .create_entity()
        .with(Transform::new([0.0, 0.0].into()))
        .with(MeshRenderComponent::simple(LineRender {
            start,
            end,
            color: Color {
                r: 0.,
                g: 1.,
                b: 0.,
                a: 1.,
            },
        }))
        .build();
    add_shape(
        world,
        e,
        Vector2::zero(),
        Segment::new(
            na::Point2::new(start.x, start.y),
            na::Point2::new(end.x, end.y),
        ),
    );
}

fn main() {
    let collision_world: PhysicsWorld = CollisionWorld::new(30.);

    let mut world = World::new();

    world.insert(DeltaTime(0.));
    world.insert(collision_world);

    world.register::<MeshRenderComponent>();
    world.register::<Collider>();
    world.register::<RoadNodeComponent>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human update", &[])
        .with(CarDecision, "car decision", &[])
        .with(
            KinematicsApply,
            "speed apply",
            &["human update", "car decision"],
        )
        .with(PhysicsUpdate, "physics", &["speed apply"])
        .with(MovableSystem::default(), "movable", &[])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world);
    cars::setup(&mut world);

    add_static_segment(&mut world, [0., 0.].into(), [10000., 0.].into());
    add_static_segment(&mut world, [0., 0.].into(), [0., 10000.].into());
    add_static_segment(&mut world, [10000., 0.].into(), [10000., 10000.].into());
    add_static_segment(&mut world, [0., 10000.].into(), [10000., 10000.].into());

    engine::start(world, dispatcher);
}
