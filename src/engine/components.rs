use cgmath::num_traits::zero;
use cgmath::Vector2;
use ggez::graphics::{Color, WHITE};
use ncollide2d::pipeline::CollisionObjectSlabHandle;
use specs::{Component, Entity, NullStorage, VecStorage};

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

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub CollisionObjectSlabHandle);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct CircleRender {
    pub radius: f32,
    pub color: Color,
    pub filled: bool,
}

impl Default for CircleRender {
    fn default() -> Self {
        CircleRender {
            radius: 0.0,
            color: WHITE,
            filled: true,
        }
    }
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
