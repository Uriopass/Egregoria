use crate::geometry::Vec2;
use cgmath::{vec4, Matrix3, SquareMatrix};
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Transform {
    m: Matrix3<f32>,
}

#[allow(dead_code)]
impl Transform {
    pub fn zero() -> Self {
        Transform::new([0.0, 0.0])
    }

    pub fn new<T: Into<Vec2>>(position: T) -> Self {
        let position = position.into();
        let mut m = Matrix3::identity();
        m.z.x = position.x;
        m.z.y = position.y;
        Transform { m }
    }

    pub fn position(&self) -> Vec2 {
        vec2!(self.m.z.x, self.m.z.y)
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.m.z.x = position.x;
        self.m.z.y = position.y;
    }

    pub fn translate(&mut self, offset: Vec2) {
        self.m.z.x += offset.x;
        self.m.z.y += offset.y;
    }

    pub fn set_angle(&mut self, angle: f32) {
        let cos = angle.cos();
        let sin = angle.sin();
        self.m.x.x = cos;
        self.m.x.y = sin;
        self.m.y.x = -sin;
        self.m.y.y = cos;
    }

    pub fn set_cos_sin(&mut self, cos: f32, sin: f32) {
        self.m.x.x = cos;
        self.m.x.y = sin;
        self.m.y.x = -sin;
        self.m.y.y = cos;
    }

    pub fn set_direction(&mut self, dir: Vec2) {
        self.set_cos_sin(dir.x, dir.y);
    }

    pub fn cos(&self) -> f32 {
        self.m.x.x
    }

    pub fn sin(&self) -> f32 {
        self.m.x.y
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

    pub fn to_matrix4(&self) -> cgmath::Matrix4<f32> {
        let m = self.m;
        cgmath::Matrix4 {
            x: vec4(m.x.x, m.x.y, 0.0, 0.0),
            y: vec4(m.y.x, m.y.y, 0.0, 0.0),
            z: vec4(0.0, 0.0, 1.0, 0.0),
            w: vec4(m.z.x, m.z.y, 0.0, 1.0),
        }
    }

    pub fn apply_rotation(&self, vec: Vec2) -> Vec2 {
        vec2!(
            vec.x * self.cos() + vec.y * self.sin(),
            vec.x * self.sin() - vec.y * self.cos(),
        )
    }

    pub fn project(&self, point: Vec2) -> Vec2 {
        let p = self.m * point.extend(1.0);
        vec2!(p.x, p.y)
    }
}
