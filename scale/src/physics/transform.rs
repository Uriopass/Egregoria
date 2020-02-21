use cgmath::{Matrix3, SquareMatrix, Vector2};
use serde::{Deserialize, Serialize};
use specs::{Component, VecStorage};

#[derive(Component, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct Transform {
    m: Matrix3<f32>,
    rotated: bool,
}

#[allow(dead_code)]
impl Transform {
    pub fn zero() -> Self {
        Transform::new([0.0, 0.0])
    }

    pub fn new<T: Into<Vector2<f32>>>(position: T) -> Self {
        let position = position.into();
        let mut m = Matrix3::identity();
        m.z.x = position.x;
        m.z.y = position.y;
        Transform { m, rotated: false }
    }

    pub fn position(&self) -> Vector2<f32> {
        Vector2::new(self.m.z.x, self.m.z.y)
    }

    pub fn set_position(&mut self, position: Vector2<f32>) {
        self.m.z.x = position.x;
        self.m.z.y = position.y;
    }

    pub fn translate(&mut self, offset: Vector2<f32>) {
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
        self.rotated = angle != 0.0;
    }

    pub fn set_cos_sin(&mut self, cos: f32, sin: f32) {
        self.m.x.x = cos;
        self.m.x.y = sin;
        self.m.y.x = -sin;
        self.m.y.y = cos;
        self.rotated = sin != 0.0;
    }

    pub fn set_direction(&mut self, dir: Vector2<f32>) {
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

    pub fn direction(&self) -> Vector2<f32> {
        Vector2::new(self.cos(), self.sin())
    }

    pub fn apply_rotation(&self, vec: Vector2<f32>) -> Vector2<f32> {
        Vector2::<f32>::new(
            vec.x * self.cos() + vec.y * self.sin(),
            vec.x * self.sin() - vec.y * self.cos(),
        )
    }

    pub fn is_angle_zero(&self) -> bool {
        !self.rotated
    }

    pub fn project(&self, point: Vector2<f32>) -> Vector2<f32> {
        let p = self.m * point.extend(1.0);
        Vector2::new(p.x, p.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn angle_test() {
        let mut x = Transform::new(Vector2::new(0.0, 0.0));
        x.set_angle(0.5);
        assert!((x.angle() - 0.5).abs() < 0.001);

        x.set_angle(0.2);
        assert!((x.angle() - 0.2).abs() < 0.001);

        x.set_angle(-0.2);
        assert!((x.angle() + 0.2).abs() < 0.001);

        x.set_angle(3.0);
        assert!((x.angle() - 3.0).abs() < 0.001);
    }
}
