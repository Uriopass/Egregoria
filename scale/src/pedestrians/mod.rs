use crate::interaction::Selectable;
use crate::physics::{Kinematics, Transform};
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use cgmath::vec2;
use specs::{Builder, World, WorldExt};

pub mod data;
pub mod systems;

pub use data::*;
pub use systems::*;

pub fn setup(world: &mut World) {
    for _ in 0..1000 {
        spawn_pedestrian(world);
    }
}

pub fn spawn_pedestrian(world: &mut World) {
    world
        .create_entity()
        .with(Transform::new(
            200.0f32 * vec2(rand::random(), rand::random()),
        ))
        .with(PedestrianComponent {
            objective: 200.0f32 * vec2(rand::random(), rand::random()),
        })
        .with(Kinematics::from_mass(80.0))
        .with(MeshRender::simple(
            CircleRender {
                radius: 0.5,
                ..Default::default()
            },
            3,
        ))
        .with(Selectable::new(0.5))
        .build();
}
