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

use crate::matrix4::Matrix4;
use crate::{vec2, Radians, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub pos: Vec3,
    pub yaw: Radians,
    pub pitch: Radians,
    pub dist: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
    up: Vec3,
    aspect: f32,
    pub fovy: f32,
}

impl Camera {
    pub fn new(pos: Vec3, viewport_w: f32, viewport_h: f32) -> Self {
        Self {
            pos,
            yaw: Radians(std::f32::consts::FRAC_PI_4),
            pitch: Radians(std::f32::consts::FRAC_PI_4),
            dist: 5000.0,
            viewport_w,
            viewport_h,
            up: (0.0, 0.0, 1.0).into(),
            aspect: viewport_w / viewport_h,
            fovy: 60.0,
        }
    }

    pub fn znear(height: f32) -> f32 {
        (1.0 + 2.0 * (height / 10.0).log10())
            .abs()
            .max(0.5)
            .min(30.0)
    }

    pub fn dir(&self) -> Vec3 {
        let v = self.yaw.vec2();
        let horiz = self.pitch.cos();
        let vert = self.pitch.sin();
        (v * horiz).z(vert)
    }

    pub fn offset(&self) -> Vec3 {
        self.dir() * self.dist / (self.fovy / 180.0 * std::f32::consts::PI).sin()
    }

    pub fn set_viewport(&mut self, w: f32, h: f32) {
        self.viewport_w = w;
        self.viewport_h = h;
        self.aspect = w / h;
    }

    pub fn eye(&self) -> Vec3 {
        self.pos + self.offset()
    }

    pub fn build_view_projection_matrix(&self) -> Matrix4 {
        let eye = self.eye();
        let znear = Self::znear(self.offset().z);
        let view = look_to_rh(eye, -self.dir(), self.up);
        let proj = PerspectiveFovReversedZ::new(
            self.fovy / 180.0 * std::f32::consts::PI,
            self.aspect,
            znear,
        )
        .mk_proj();

        proj * view
    }

    pub fn build_sun_shadowmap_matrix(&self, mut dir: Vec3, resolution: f32) -> Matrix4 {
        if dir.x == 0.0 && dir.y == 0.0 {
            dir.x = 0.01;
            dir.y = 0.01;
        }

        let d = self.dist * 2.5;

        let base = self.pos;

        let view = look_at_rh(base + dir, base, self.up);
        let proj: Matrix4 = Ortho {
            left: -d,
            right: d,
            bottom: -d,
            top: d,
            near: -d * 1.2,
            far: d * 1.2,
        }
        .into();
        // texel snapping
        let projview = proj * view;

        let proj_base = projview * Vec4::from([0.0, 0.0, 0.0, 1.0]);

        let texcoord = vec2(proj_base.x, proj_base.y) * 0.5 * resolution;

        let rounded = (texcoord + vec2(0.5, 0.5)).floor();

        let dtex = rounded - texcoord;

        let dtex_orig = dtex / (0.5 * resolution);

        let rounding = Matrix4::from([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [dtex_orig.x, dtex_orig.y, 0.0, 1.0],
        ]);

        opengl_to_wgpu_matrix() * (rounding * projview)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PerspectiveFovReversedZ {
    pub fovy_angle: f32, // Angle
    pub aspect: f32,
    pub near: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ortho {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

#[rustfmt::skip]
pub fn opengl_to_wgpu_matrix() -> Matrix4 {
    Matrix4::from([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    ])
}

pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Matrix4 {
    look_to_rh(eye, center - eye, up)
}

/// Create a homogeneous transformation matrix that will cause a vector to point at
/// `dir`, using `up` for orientation.
#[rustfmt::skip]
pub fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Matrix4 {
    let f = dir.normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);

    Matrix4::from([
        s.x,             u.x,              -f.x,           0.0,
        s.y,             u.y,              -f.y,           0.0,
        s.z,             u.z,              -f.z,           0.0,
        -eye.dot(s), -eye.dot(u), eye.dot(f), 1.0,
    ])
}

#[rustfmt::skip]
impl From<Ortho> for Matrix4 {
    fn from(ortho: Ortho) -> Self {
        let c0r0 = 2.0 / (ortho.right - ortho.left);
        let c0r1 = 0.0;
        let c0r2 = 0.0;
        let c0r3 = 0.0;

        let c1r0 = 0.0;
        let c1r1 = 2.0 / (ortho.top - ortho.bottom);
        let c1r2 = 0.0;
        let c1r3 = 0.0;

        let c2r0 = 0.0;
        let c2r1 = 0.0;
        let c2r2 = -2.0 / (ortho.far - ortho.near);
        let c2r3 = 0.0;

        let c3r0 = -(ortho.right + ortho.left) / (ortho.right - ortho.left);
        let c3r1 = -(ortho.top + ortho.bottom) / (ortho.top - ortho.bottom);
        let c3r2 = -(ortho.far + ortho.near) / (ortho.far - ortho.near);
        let c3r3 = 1.0;

        Matrix4::from([
            c0r0, c0r1, c0r2, c0r3,
            c1r0, c1r1, c1r2, c1r3,
            c2r0, c2r1, c2r2, c2r3,
            c3r0, c3r1, c3r2, c3r3,
            ]
        )
    }
}

impl PerspectiveFovReversedZ {
    pub fn new(fovy_angle: f32, aspect: f32, near: f32) -> Self {
        PerspectiveFovReversedZ {
            fovy_angle,
            aspect,
            near,
        }
    }
}

impl PerspectiveFovReversedZ {
    #[rustfmt::skip]
    pub fn mk_proj(&self) -> Matrix4 {
        assert!(
            self.fovy_angle > 0.0,
            "The vertical field of view cannot be below zero, found: {:?}",
            self.fovy_angle
        );
        assert!(
            self.fovy_angle < std::f32::consts::PI,
            "The vertical field of view cannot be greater than a turn, found: {:?}",
            self.fovy_angle
        );
        assert!(
            self.aspect > 0.0,
            "The aspect ratio cannot be below zero, found: {:?}",
            self.aspect
        );
        assert!(
            self.near > 0.0,
            "The near plane distance cannot be below zero, found: {:?}",
            self.near
        );
        /*
        let f = 1.0 / (self.fovy_angle / 2.0).tan();

        let a = (self.far + self.near) / (self.near - self.far);
        let b = 2.0*self.far*self.near / (self.near - self.far);

        let c0 = [f / self.aspect, 0.0, 0.0, 0.0];
        let c1 = [0.0            , f  , 0.0, 0.0];
        let c2 = [0.0            , 0.0, a,  -1.0];
        let c3 = [0.0            , 0.0, b,   0.0];
        */

        let f = 1.0 / (self.fovy_angle / 2.0).tan();

        let c0 = [f / self.aspect, 0.0, 0.0, 0.0];
        let c1 = [0.0            , f  , 0.0, 0.0];
        let c2 = [0.0            , 0.0, -0.5,  -1.0];
        let c3 = [0.0            , 0.0, self.near,   0.0];

        Matrix4::from([c0, c1, c2, c3])
    }
}
