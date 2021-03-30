use crate::{vec2, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    position: Vec2,
    cossin: Vec2,
}

impl Transform {
    pub fn zero() -> Self {
        Transform::new([0.0, 0.0])
    }

    pub fn new<T: Into<Vec2>>(position: T) -> Self {
        let position = position.into();
        Transform {
            position,
            cossin: Vec2::UNIT_X,
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
        vec2(self.cos(), self.sin())
    }

    pub fn normal(&self) -> Vec2 {
        vec2(-self.sin(), self.cos())
    }

    pub fn to_matrix4(&self, z: f32) -> mint::ColumnMatrix4<f32> {
        mint::ColumnMatrix4 {
            x: [self.cossin.x, self.cossin.y, 0.0, 0.0].into(),
            y: [-self.cossin.y, self.cossin.x, 0.0, 0.0].into(),
            z: [0.0, 0.0, 0.0, 0.0].into(),
            w: [self.position.x, self.position.y, z, 1.0].into(),
        }
    }

    pub fn apply_rotation(&self, point: Vec2) -> Vec2 {
        point.rotated_by(self.cossin)
    }

    pub fn project(&self, point: Vec2) -> Vec2 {
        point.rotated_by(self.cossin) + self.position
    }
}
