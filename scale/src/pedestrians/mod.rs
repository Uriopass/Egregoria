use crate::interaction::Selectable;
use crate::physics::Transform;
use crate::rendering::meshrender_component::{CircleRender, MeshRender};
use cgmath::Vector2;
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
            200.0 * Vector2::<f32>::from(rand::random::<[f32; 2]>()),
        ))
        .with(PedestrianComponent {
            objective: rand::random::<[f32; 2]>().into(),
        })
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
