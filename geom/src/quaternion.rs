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

impl From<[f32; 4]> for Quaternion {
    #[inline]
    fn from(x: [f32; 4]) -> Self {
        unsafe { std::mem::transmute(x) }
    }
}
