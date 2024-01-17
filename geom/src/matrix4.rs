// Copyright 2013-2014 The CGMath Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Modified for the Egregoria project by the Egregoria developers.

use crate::Vec4;
use std::ops::Mul;

/// Column major matrix
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct Matrix4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub w: Vec4,
}

impl Matrix4 {
    pub fn zero() -> Self {
        Self::default()
    }

    pub fn determinent(&self) -> f32 {
        let tmp0 = unsafe { det_sub_proc_unsafe(self, 1, 2, 3) };
        tmp0.dot(&Vec4::from([self.x.x, self.y.x, self.z.x, self.w.x]))
    }

    pub fn invert(&self) -> Option<Matrix4> {
        let tmp0 = unsafe { det_sub_proc_unsafe(self, 1, 2, 3) };
        let det = tmp0.dot(&Vec4::from([self.x.x, self.y.x, self.z.x, self.w.x]));
        if det.abs() < f32::EPSILON {
            None
        } else {
            let inv_det = 1.0 / det;
            let x = tmp0 * inv_det;
            let y = unsafe { det_sub_proc_unsafe(self, 0, 3, 2) * inv_det };
            let z = unsafe { det_sub_proc_unsafe(self, 0, 1, 3) * inv_det };
            let w = unsafe { det_sub_proc_unsafe(self, 0, 2, 1) * inv_det };
            Some(Matrix4 { x, y, z, w })
        }
    }
}

impl Mul<Vec4> for Matrix4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }
}

impl<'a> Mul<Vec4> for &'a Matrix4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }
}

impl Mul for Matrix4 {
    type Output = Matrix4;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Matrix4::from([
            self.x * rhs.x.x + self.y * rhs.x.y + self.z * rhs.x.z + self.w * rhs.x.w,
            self.x * rhs.y.x + self.y * rhs.y.y + self.z * rhs.y.z + self.w * rhs.y.w,
            self.x * rhs.z.x + self.y * rhs.z.y + self.z * rhs.z.z + self.w * rhs.z.w,
            self.x * rhs.w.x + self.y * rhs.w.y + self.z * rhs.w.z + self.w * rhs.w.w,
        ])
    }
}

impl From<[f32; 16]> for Matrix4 {
    fn from(x: [f32; 16]) -> Self {
        unsafe { std::mem::transmute(x) }
    }
}

impl From<[[f32; 4]; 4]> for Matrix4 {
    fn from(x: [[f32; 4]; 4]) -> Self {
        unsafe { std::mem::transmute(x) }
    }
}

impl From<[Vec4; 4]> for Matrix4 {
    fn from(x: [Vec4; 4]) -> Self {
        unsafe { std::mem::transmute(x) }
    }
}

impl AsRef<[f32; 16]> for Matrix4 {
    fn as_ref(&self) -> &[f32; 16] {
        unsafe { &*(self as *const Matrix4 as *const [f32; 16]) }
    }
}

unsafe fn det_sub_proc_unsafe(m: &Matrix4, x: usize, y: usize, z: usize) -> Vec4 {
    let s: &[f32; 16] = m.as_ref();
    let a = Vec4::from([
        *s.get_unchecked(4 + x),
        *s.get_unchecked(12 + x),
        *s.get_unchecked(x),
        *s.get_unchecked(8 + x),
    ]);
    let b = Vec4::from([
        *s.get_unchecked(8 + y),
        *s.get_unchecked(8 + y),
        *s.get_unchecked(4 + y),
        *s.get_unchecked(4 + y),
    ]);
    let c = Vec4::from([
        *s.get_unchecked(12 + z),
        *s.get_unchecked(z),
        *s.get_unchecked(12 + z),
        *s.get_unchecked(z),
    ]);

    let d = Vec4::from([
        *s.get_unchecked(8 + x),
        *s.get_unchecked(8 + x),
        *s.get_unchecked(4 + x),
        *s.get_unchecked(4 + x),
    ]);
    let e = Vec4::from([
        *s.get_unchecked(12 + y),
        *s.get_unchecked(y),
        *s.get_unchecked(12 + y),
        *s.get_unchecked(y),
    ]);
    let f = Vec4::from([
        *s.get_unchecked(4 + z),
        *s.get_unchecked(12 + z),
        *s.get_unchecked(z),
        *s.get_unchecked(8 + z),
    ]);

    let g = Vec4::from([
        *s.get_unchecked(12 + x),
        *s.get_unchecked(x),
        *s.get_unchecked(12 + x),
        *s.get_unchecked(x),
    ]);
    let h = Vec4::from([
        *s.get_unchecked(4 + y),
        *s.get_unchecked(12 + y),
        *s.get_unchecked(y),
        *s.get_unchecked(8 + y),
    ]);
    let i = Vec4::from([
        *s.get_unchecked(8 + z),
        *s.get_unchecked(8 + z),
        *s.get_unchecked(4 + z),
        *s.get_unchecked(4 + z),
    ]);
    let mut tmp = a * b * c;
    tmp += d * e * f;
    tmp += g * h * i;
    tmp += -a * e * i;
    tmp += -d * h * c;
    tmp += -g * b * f;
    tmp
}
