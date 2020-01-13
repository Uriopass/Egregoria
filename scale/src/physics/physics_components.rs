use cgmath::num_traits::zero;
use cgmath::{Matrix3, SquareMatrix, Vector2};
use ncollide2d::pipeline::CollisionObjectSlabHandle;
use specs::{Component, VecStorage};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub CollisionObjectSlabHandle);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Drag(pub f32);

const DRAG_COEFF: f32 = 0.1;

impl Drag {
    pub fn new(coeff: f32) -> Self {
        Drag(coeff)
    }
}
impl Default for Drag {
    fn default() -> Self {
        Drag(DRAG_COEFF)
    }
}

#[derive(Component, Debug, PartialEq, Clone)]
#[storage(VecStorage)]
pub struct Transform {
    m: Matrix3<f32>,
    rotated: bool,
}

#[allow(dead_code)]
impl Transform {
    pub fn new<T: Into<Vector2<f32>>>(position: T) -> Self {
        let position = position.into();
        let mut m = Matrix3::identity();
        m.z.x = position.x;
        m.z.y = position.y;
        Transform { m, rotated: false }
    }

    pub fn get_position(&self) -> Vector2<f32> {
        self.m.z.xy()
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

    pub fn set_angle_cos_sin(&mut self, cos: f32, sin: f32) {
        self.m.x.x = cos;
        self.m.x.y = sin;
        self.m.y.x = -sin;
        self.m.y.y = cos;
        self.rotated = sin != 0.0;
    }

    pub fn get_cos(&self) -> f32 {
        self.m.x.x
    }

    pub fn get_sin(&self) -> f32 {
        self.m.x.y
    }

    pub fn get_angle(&self) -> f32 {
        f32::atan2(self.get_sin(), self.get_cos())
    }

    pub fn is_angle_zero(&self) -> bool {
        !self.rotated
    }

    pub fn project(&self, point: Vector2<f32>) -> Vector2<f32> {
        (self.m * point.extend(1.0)).xy()
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Kinematics {
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
    pub mass: f32,
}

impl Kinematics {
    pub fn from_mass(mass: f32) -> Self {
        Kinematics {
            velocity: zero(),
            acceleration: zero(),
            mass,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn angle_test() {
        let mut x = Transform::new(Vector2::new(0.0, 0.0));
        x.set_angle(0.5);
        assert!((x.get_angle() - 0.5).abs() < 0.001);

        x.set_angle(0.2);
        assert!((x.get_angle() - 0.2).abs() < 0.001);

        x.set_angle(-0.2);
        assert!((x.get_angle() + 0.2).abs() < 0.001);

        x.set_angle(3.0);
        assert!((x.get_angle() - 3.0).abs() < 0.001);
    }
}
