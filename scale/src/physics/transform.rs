use crate::geometry::Vec2;
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Transform {
    position: Vec2,
    cossin: Vec2,
}

#[allow(dead_code)]
impl Transform {
    pub fn zero() -> Self {
        Transform::new([0.0, 0.0])
    }

    pub fn new<T: Into<Vec2>>(position: T) -> Self {
        let position = position.into();
        Transform {
            position,
            cossin: vec2!(1.0, 0.0),
        }
    }

    pub fn new_cos_sin(position: Vec2, cossin: Vec2) -> Self {
        Transform { position, cossin }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn translate(&mut self, offset: Vec2) {
        self.position += offset;
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.cossin.x = angle.cos();
        self.cossin.y = angle.sin();
    }

    pub fn set_cos_sin(&mut self, cos: f32, sin: f32) {
        self.cossin.x = cos;
        self.cossin.y = sin;
    }

    pub fn set_direction(&mut self, dir: Vec2) {
        self.cossin = dir;
    }

    pub fn cos(&self) -> f32 {
        self.cossin.x
    }

    pub fn sin(&self) -> f32 {
        self.cossin.y
    }

    pub fn angle(&self) -> f32 {
        f32::atan2(self.sin(), self.cos())
    }

    pub fn direction(&self) -> Vec2 {
        vec2!(self.cos(), self.sin())
    }

    pub fn normal(&self) -> Vec2 {
        vec2!(-self.sin(), self.cos())
    }

    pub fn to_matrix4(&self, z: f32) -> mint::ColumnMatrix4<f32> {
        mint::ColumnMatrix4 {
            x: [self.cossin.x, self.cossin.y, 0.0, 0.0].into(),
            y: [-self.cossin.y, self.cossin.x, 0.0, 0.0].into(),
            z: [0.0, 0.0, 0.0, 0.0].into(),
            w: [self.position.x, self.position.y, z, 1.0].into(),
        }
    }

    pub fn apply_rotation(&self, vec: Vec2) -> Vec2 {
        vec2!(
            vec.x * self.cos() + vec.y * self.sin(),
            vec.x * self.sin() - vec.y * self.cos(),
        )
    }

    pub fn project(&self, point: Vec2) -> Vec2 {
        let rotated = point * self.cossin + point * vec2!(-self.cossin.y, self.cossin.x);
        rotated + self.position
    }
}
