use cgmath::Vector2;
use ggez::input::mouse::{MouseButton, MouseContext};
use std::collections::HashSet;

#[derive(Default)]
pub struct DeltaTime(pub f32);

pub struct MouseInfo {
    pub unprojected: Vector2<f32>,
    pub buttons: HashSet<MouseButton>,
}

impl Default for MouseInfo {
    fn default() -> Self {
        MouseInfo {
            unprojected: [0., 0.].into(),
            buttons: HashSet::new(),
        }
    }
}
