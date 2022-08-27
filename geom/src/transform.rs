use crate::{vec2, Matrix4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

const UP: Vec3 = Vec3::Z;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub dir: Vec3,
}

impl Transform {
    #[inline]
    pub fn zero() -> Self {
        Self::new(Vec3::ZERO)
    }

    pub fn new<T: Into<Vec3>>(position: T) -> Self {
        let position = position.into();
        Self {
            position,
            dir: Vec3::X,
        }
    }

    #[inline]
    pub fn new_dir(position: Vec3, dir: Vec3) -> Self {
        Self { position, dir }
    }

    #[inline]
    pub fn angle(&self) -> f32 {
        f32::atan2(self.dir.y, self.dir.x)
    }

    #[inline]
    pub fn normalxy(&self) -> Vec2 {
        vec2(-self.dir.y, self.dir.x)
    }

    pub fn to_matrix4(&self) -> Matrix4 {
        let x = self.dir;
        let y = self.dir.cross(UP).try_normalize().unwrap_or(Vec3::Y);
        let z = x.cross(y).try_normalize().unwrap_or(Vec3::Z);

        Matrix4 {
            x: x.w(0.0),
            y: y.w(0.0),
            z: z.w(0.0),
            w: self.position.w(1.0),
        }
    }

    #[inline]
    pub fn apply_rotation(&self, point: Vec3) -> Vec3 {
        point.rotate_up(self.dir)
    }

    #[inline]
    pub fn project(&self, point: Vec3) -> Vec3 {
        point.rotate_up(self.dir) + self.position
    }
}
