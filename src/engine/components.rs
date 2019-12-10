use cgmath::Vector2;
use ggez::graphics::Color;
use ncollide2d::pipeline::CollisionObjectSlabHandle;
use specs::{Component, Entity, NullStorage, VecStorage};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position(pub cgmath::Vector2<f32>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity(pub cgmath::Vector2<f32>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub CollisionObjectSlabHandle);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CircleRender {
    pub radius: f32,
    pub color: Color,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct LineToRender {
    pub to: Entity,
    pub color: Color,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct LineRender {
    pub start: Vector2<f32>,
    pub end: Vector2<f32>,
    pub color: Color,
}

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Movable;
