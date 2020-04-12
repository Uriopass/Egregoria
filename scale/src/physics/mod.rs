use crate::geometry::gridstore::{GridStore, GridStoreHandle};
use cgmath::{vec2, Vector2};
use specs::{Component, VecStorage};

mod kinematics;
pub mod systems;
mod transform;

pub use kinematics::*;
pub use transform::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PhysicsGroup {
    Unknown,
    Vehicles,
    Pedestrians,
}

#[derive(Clone, Copy)]
pub struct PhysicsObject {
    pub dir: Vector2<f32>,
    pub speed: f32,
    pub radius: f32,
    pub group: PhysicsGroup,
}

impl Default for PhysicsObject {
    fn default() -> Self {
        Self {
            dir: vec2(1.0, 0.0),
            speed: 0.0,
            radius: 1.0,
            group: PhysicsGroup::Unknown,
        }
    }
}

pub type CollisionWorld = GridStore<PhysicsObject>;

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub GridStoreHandle);
