use std::env;
use std::path;

use crate::components::{Collider, LineRender, MeshRenderComponent, Movable, Transform};
use cgmath::Vector2;
use ggez::conf::NumSamples;
use ggez::{conf, event, ContextBuilder};
use ncollide2d::pipeline::{CollisionGroups, GeometricQueryType};
use ncollide2d::shape::{Segment, Shape, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use specs::{Builder, Dispatcher, Entity, World, WorldExt};

pub mod components;
pub mod game_loop;
pub mod geometry;
pub mod rendering;
pub mod resources;
pub mod systems;

pub use cgmath;
pub use nalgebra;
pub use ncollide2d;
pub use rendering::colors::*;
pub use specs;

const PHYSICS_UPDATES: usize = 2;
pub type PhysicsWorld = CollisionWorld<f32, Entity>;

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
        .with(MeshRenderComponent::simple(LineRender {
            offset,
            color: GREEN,
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

pub fn start<'a>(world: World, schedule: Dispatcher<'a, 'a>) {
    let mut c = conf::Conf::new();
    if cfg!(target_os = "windows") {
        c.window_mode = c.window_mode.dimensions(1600.0, 900.0);
    } else {
        c.window_mode = c.window_mode.dimensions(800.0, 600.0);
    }
    c.window_setup = c.window_setup.vsync(false).samples(NumSamples::Four);

    let mut cb = ContextBuilder::new("Sandbox", "Uriopass").conf(c);

    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }

    let (ref mut ctx, ref mut event_loop) = cb.build().unwrap();

    let mut state = game_loop::EngineState::new(world, schedule, ctx).unwrap();

    state.cam.camera.zoom = 10.0;
    state.cam.camera.position.x = 50.0;
    state.cam.camera.position.y = 50.0;

    event::run(ctx, event_loop, &mut state).unwrap()
}
