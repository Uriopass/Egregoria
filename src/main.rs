use engine::*;

use crate::engine::components::{CircleRender, Collider, LineRender, LineToRender, Position};
use crate::engine::resources::DeltaTime;
use crate::engine::systems::{MovableSystem, PhysicsUpdate, SpeedApply};
use crate::humans::HumanUpdate;
use cgmath::{Vector2, Zero};
use ggez::graphics::Color;
use ncollide2d::shape::{Segment, Shape, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use specs::prelude::*;

mod dijkstra;
mod engine;
mod geometry;
mod humans;

use nalgebra as na;
use ncollide2d::pipeline::{CollisionGroups, GeometricQueryType};

type PhysicsWorld = CollisionWorld<f32, Entity>;

pub fn add_shape<T>(
    coworld: &mut PhysicsWorld,
    world: &mut World,
    e: Entity,
    pos: Vector2<f32>,
    shape: T,
) where
    T: Shape<f32>,
{
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

fn main() {
    let mut collision_world: PhysicsWorld = CollisionWorld::new(1.);

    let mut world = World::new();

    world.insert(DeltaTime(0.));

    world.register::<CircleRender>();
    world.register::<LineToRender>();
    world.register::<LineRender>();
    world.register::<Collider>();

    let mut dispatcher = DispatcherBuilder::new()
        .with(HumanUpdate, "human_update", &[])
        .with(SpeedApply, "speed_apply", &["human_update"])
        .with(PhysicsUpdate, "physics", &["speed_apply"])
        .with(MovableSystem::default(), "movable", &[])
        .build();

    dispatcher.setup(&mut world);

    humans::setup(&mut world, &mut collision_world);

    let e = world
        .create_entity()
        .with(Position([0.0, 0.0].into()))
        .with(LineRender {
            start: Vector2::new(0., 0.),
            end: Vector2::new(1000., 1000.),
            color: Color {
                r: 0.,
                g: 1.,
                b: 0.,
                a: 1.,
            },
        })
        .build();
    add_shape(
        &mut collision_world,
        &mut world,
        e,
        Vector2::zero(),
        Segment::new(na::Point2::new(0., 0.), na::Point2::new(1000., 1000.)),
    );

    let e = world
        .create_entity()
        .with(Position([0.0, 0.0].into()))
        .with(LineRender {
            start: Vector2::new(0., 0.),
            end: Vector2::new(-1000., 1000.),
            color: Color {
                r: 0.,
                g: 1.,
                b: 0.,
                a: 1.,
            },
        })
        .build();

    add_shape(
        &mut collision_world,
        &mut world,
        e,
        Vector2::zero(),
        Segment::new(na::Point2::new(0., 0.), na::Point2::new(-1000., 1000.)),
    );

    world.insert::<PhysicsWorld>(collision_world);
    engine::start(world, dispatcher);
}
