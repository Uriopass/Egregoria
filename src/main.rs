use engine::*;

use crate::engine::components::{
    CircleRender, Collider, LineRender, LineToRender, Position, RectRender,
};
use crate::engine::resources::DeltaTime;
use crate::engine::systems::{KinematicsApply, MovableSystem, PhysicsUpdate};
use crate::humans::HumanUpdate;
use cgmath::{Vector2, Zero};
use ggez::graphics::Color;
use ncollide2d::shape::{Segment, Shape, ShapeHandle};
use ncollide2d::world::CollisionWorld;

mod dijkstra;
mod engine;
mod geometry;
mod humans;

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

pub fn add_segment(world: &mut World, start: Vector2<f32>, end: Vector2<f32>) {
    let e = world
        .create_entity()
        .with(Position([0.0, 0.0].into()))
        .with(LineRender {
            start,
            end,
            color: Color {
                r: 0.,
                g: 1.,
                b: 0.,
                a: 1.,
            },
        })
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
    let mut collision_world: PhysicsWorld = CollisionWorld::new(1.);

    let mut world = World::new();

    world.insert(DeltaTime(0.));
    world.insert::<PhysicsWorld>(collision_world);

    world.register::<CircleRender>();
    world.register::<RectRender>();
    world.register::<LineToRender>();
    world.register::<LineRender>();
    world.register::<Collider>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human_update", &[])
        .with(KinematicsApply, "speed_apply", &["human_update"])
        .with(PhysicsUpdate, "physics", &["speed_apply"])
        .with(MovableSystem::default(), "movable", &[])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world);

    add_segment(&mut world, [0.0, 0.0].into(), [1000., 0.].into());

    add_segment(&mut world, [0.0, 0.0].into(), [0., 400.].into());

    add_segment(&mut world, [1000.0, 0.0].into(), [1000., 400.].into());
    add_segment(&mut world, [0., 400.0].into(), [1000., 400.].into());

    engine::start(world, dispatcher);
}
