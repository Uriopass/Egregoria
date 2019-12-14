use cgmath::num_traits::zero;
use cgmath::Vector2;
use specs::{Component, VecStorage};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Drag(pub f32);

const DRAG_COEFF: f32 = 0.2;
impl Default for Drag {
    fn default() -> Self {
        Drag(DRAG_COEFF)
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position(pub Vector2<f32>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Kinematics {
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
}

impl Kinematics {
    pub fn zero() -> Self {
        Kinematics {
            velocity: zero(),
            acceleration: zero(),
        }
    }

    #[allow(dead_code)]
    pub fn from_velocity(x: Vector2<f32>) -> Self {
        Kinematics {
            velocity: x,
            acceleration: zero(),
        }
    }
}
