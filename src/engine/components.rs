#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Position(pub cgmath::Vector2<f32>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Velocity(pub cgmath::Vector2<f32>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CircleRender {
    pub radius: f32,
}
