use crate::Vec3;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub};

#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[repr(C)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl Vec4 {
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
        w: 0.0,
    };

    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };

    pub const W: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub fn dot(&self, other: &Vec4) -> f32 {
        (self.x * other.x + self.y * other.y) + (self.z * other.z + self.w * other.w)
    }

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn new_w(v: Vec3, w: f32) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w,
        }
    }

    pub fn xyz(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl From<[f32; 4]> for Vec4 {
    #[inline]
    fn from(x: [f32; 4]) -> Self {
        unsafe { std::mem::transmute(x) }
    }
}

impl From<Vec4> for [f32; 4] {
    #[inline]
    fn from(x: Vec4) -> Self {
        unsafe { std::mem::transmute(x) }
    }
}

impl Eq for Vec4 {}

#[inline]
pub const fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 {
    Vec4 { x, y, z, w }
}

impl Add for Vec4 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Add for &Vec4 {
    type Output = Vec4;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Add<Vec4> for &Vec4 {
    type Output = Vec4;

    #[inline]
    fn add(self, rhs: Vec4) -> Self::Output {
        Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Add<&Vec4> for Vec4 {
    type Output = Vec4;

    #[inline]
    fn add(self, rhs: &Vec4) -> Self::Output {
        Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl AddAssign for Vec4 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += rhs.w;
    }
}

impl Sub for Vec4 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl Sub for &Vec4 {
    type Output = Vec4;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vec4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    #[inline]
    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
            w: self * rhs.w,
        }
    }
}

impl Mul<f32> for Vec4 {
    type Output = Vec4;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl Mul<Vec4> for Vec4 {
    type Output = Vec4;

    #[inline]
    fn mul(self, rhs: Vec4) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
            w: self.w * rhs.w,
        }
    }
}

impl MulAssign for Vec4 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Div<Vec4> for f32 {
    type Output = Vec4;

    #[inline]
    fn div(self, rhs: Vec4) -> Self::Output {
        Vec4 {
            x: self / rhs.x,
            y: self / rhs.y,
            z: self / rhs.z,
            w: self / rhs.w,
        }
    }
}

impl Div<f32> for Vec4 {
    type Output = Vec4;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
            w: self.w / rhs,
        }
    }
}

impl Div<Vec4> for Vec4 {
    type Output = Vec4;

    #[inline]
    fn div(self, rhs: Vec4) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
            w: self.w / rhs.w,
        }
    }
}

impl Neg for Vec4 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}
