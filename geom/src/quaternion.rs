use crate::{vec3, Vec3};
use std::ops::Mul;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Mul<Vec3> for Quaternion {
    type Output = Vec3;

    #[inline]
    fn mul(self, v: Vec3) -> Vec3 {
        let qv = vec3(self.x, self.y, self.z);
        let qs = self.w;
        2.0 * qv.dot(v) * qv + (qs * qs - qv.dot(qv)) * v + 2.0 * qs * qv.cross(v)
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    #[inline]
    fn mul(self, rhs: Quaternion) -> Quaternion {
        let q1 = vec3(self.x, self.y, self.z);
        let q2 = vec3(rhs.x, rhs.y, rhs.z);
        let w1 = self.w;
        let w2 = rhs.w;
        Quaternion {
            x: w1 * q2.x + w2 * q1.x + q1.cross(q2).x,
            y: w1 * q2.y + w2 * q1.y + q1.cross(q2).y,
            z: w1 * q2.z + w2 * q1.z + q1.cross(q2).z,
            w: w1 * w2 - q1.dot(q2),
        }
    }
}

impl From<[f32; 4]> for Quaternion {
    #[inline]
    fn from(x: [f32; 4]) -> Self {
        unsafe { std::mem::transmute(x) }
    }
}
