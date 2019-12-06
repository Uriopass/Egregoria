use specs::{Component, VecStorage};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position(pub cgmath::Vector2<f32>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Velocity(pub cgmath::Vector2<f32>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CircleRender {
    pub radius: f32,
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Movable;
