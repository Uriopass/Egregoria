use cgmath::Vector2;
use ggez::input::keyboard::KeyCode;
use ggez::input::mouse::MouseButton;
use std::collections::HashSet;

#[derive(Default)]
pub struct DeltaTime(pub f32);

pub struct MouseInfo {
    pub unprojected: Vector2<f32>,
    pub buttons: HashSet<MouseButton>,
    pub just_pressed: HashSet<MouseButton>,
}

impl Default for MouseInfo {
    fn default() -> Self {
        MouseInfo {
            unprojected: [0.0, 0.0].into(),
            buttons: HashSet::new(),
            just_pressed: HashSet::new(),
        }
    }
}

pub struct KeyboardInfo {
    pub just_pressed: HashSet<KeyCode>,
}

impl Default for KeyboardInfo {
    fn default() -> Self {
        KeyboardInfo {
            just_pressed: HashSet::new(),
        }
    }
}
