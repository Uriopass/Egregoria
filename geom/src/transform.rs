use crate::{vec2, Matrix4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

const UP: Vec3 = Vec3::Z;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transform {
    pub pos: Vec3,
    pub dir: Vec3,
}

impl Transform {
    #[inline]
    pub fn zero() -> Self {
        Self::new(Vec3::ZERO)
    }

    pub fn new<T: Into<Vec3>>(pos: T) -> Self {
        let pos = pos.into();
        Self { pos, dir: Vec3::X }
    }

    #[inline]
    pub fn new_dir(pos: Vec3, dir: Vec3) -> Self {
        Self { pos, dir }
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
            w: self.pos.w(1.0),
        }
    }

    #[inline]
    pub fn apply_rotation(&self, point: Vec3) -> Vec3 {
        point.rotate_up(self.dir)
    }

    #[inline]
    pub fn project(&self, point: Vec3) -> Vec3 {
        point.rotate_up(self.dir) + self.pos
    }
}

#[cfg(test)]
mod tests {
    use crate::{vec3, Transform, Vec3};

    #[test]
    fn test_rotation_matrix_is_same() {
        let t = Transform::new_dir(Vec3::ZERO, Vec3::Y);
        let m = t.to_matrix4();

        for v in [
            vec3(1.0, 1.0, 1.0),
            vec3(1.0, 1.0, -1.0),
            vec3(1.0, -1.0, 1.0),
            vec3(1.0, -1.0, -1.0),
            vec3(-1.0, 1.0, 1.0),
            vec3(-1.0, 1.0, -1.0),
            vec3(-1.0, -1.0, 1.0),
            vec3(-1.0, -1.0, -1.0),
        ] {
            assert_eq!(t.apply_rotation(v), (m * v.w(0.0)).xyz(), "{}", v);
        }
    }
}
