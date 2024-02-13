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
use crate::{vec2, vec3, vec4, InfiniteFrustrum, Radians, Ray3, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub pos: Vec3,
    pub yaw: Radians,
    pub pitch: Radians,
    pub dist: f32,       // in meters
    pub viewport_w: f32, // in pixels
    pub viewport_h: f32, // in pixels
    up: Vec3,
    aspect: f32,
    pub fovy: f32,
    #[serde(default, skip)]
    pub proj_cache: Matrix4,
    #[serde(default, skip)]
    pub inv_proj_cache: Matrix4,
}

impl Camera {
    pub fn new(pos: Vec3, viewport_w: f32, viewport_h: f32) -> Self {
        Self {
            pos,
            yaw: Radians(-0.21086383),
            pitch: Radians(0.8478442),
            dist: 932.0,
            viewport_w,
            viewport_h,
            up: (0.0, 0.0, 1.0).into(),
            aspect: viewport_w / viewport_h,
            fovy: 60.0,
            proj_cache: Matrix4::zero(),
            inv_proj_cache: Matrix4::zero(),
        }
    }

    pub fn dir(&self) -> Vec3 {
        let v = self.yaw.vec2();
        let horiz = self.pitch.cos();
        let vert = self.pitch.sin();
        (v * horiz).z(vert).normalize()
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

    pub fn unproj_ray(&self, pos: Vec2) -> Option<Ray3> {
        let v = self.inv_proj_cache
            * vec4(
                2.0 * pos.x / self.viewport_w - 1.0,
                -(2.0 * pos.y / self.viewport_h - 1.0),
                1.0,
                1.0,
            );

        let v = Vec3 {
            x: v.x / v.w,
            y: v.y / v.w,
            z: v.z / v.w,
        } - self.eye();
        Some(Ray3 {
            from: self.eye(),
            dir: v.normalize(),
        })
    }

    /// Project a 3D point to the screen
    /// Returns the screen position (in viewport space) and the depth (inverse z)
    pub fn project(&self, pos: Vec3) -> (Vec2, f32) {
        let v = self.proj_cache * vec4(pos.x, pos.y, pos.z, 1.0);
        let v = Vec3 {
            x: v.x / v.w,
            y: v.y / v.w,
            z: v.z / v.w,
        };

        let x = (v.x + 1.0) * 0.5 * self.viewport_w;
        let y = (1.0 - v.y) * 0.5 * self.viewport_h;
        let depth = 1.0 / v.z; // we use reverse z

        (vec2(x, y), depth)
    }

    pub fn update(&mut self) {
        self.proj_cache = self.build_view_projection_matrix();
        self.inv_proj_cache = self.proj_cache.invert().unwrap_or_else(Matrix4::zero);
    }

    fn build_view_projection_matrix(&self) -> Matrix4 {
        let eye = self.eye();
        let view = look_to_rh(eye, -self.dir(), self.up);
        let proj = PerspectiveFovReversedZ::new(
            self.fovy / 180.0 * std::f32::consts::PI,
            self.aspect,
            1.0,
        )
        .mk_proj();

        proj * view
    }

    pub fn build_sun_shadowmap_matrix(
        &self,
        mut dir: Vec3,
        resolution: f32,
        _frustrum: &InfiniteFrustrum,
    ) -> Vec<Matrix4> {
        if dir.x == 0.0 && dir.y == 0.0 {
            dir.x = 0.01;
            dir.y = 0.01;
        }
        let center_cam = self.pos;

        let mut cascades = Vec::with_capacity(4);

        let m = (self.dist.floor() * 0.5).min(100.0).max(10.0);

        let z_down = -10.0;
        let z_up = 30.0 + m;

        //        for cascade in [0.0, 300.0, 2000.0, 30000.0].windows(2) {
        for dist in [m * 2.5, m * 6.25, m * 15.5, m * 50.0] {
            /*
            let cascade: [f32; 2] = cascade.try_into().unwrap();
            let c = frustrum.create_cascade(cascade[0], cascade[1]);

            let mut center = c.points.iter().sum::<Vec3>() / c.points.len() as f32;
             */

            //            let mut points = &c.points;
            //            let v;
            //            if cascade[1] == 30000.0 {
            let v = [
                center_cam + vec3(-dist, -dist, z_down),
                center_cam + vec3(dist, -dist, z_down),
                center_cam + vec3(dist, dist, z_down),
                center_cam + vec3(-dist, dist, z_down),
                center_cam + vec3(-dist, -dist, z_up),
                center_cam + vec3(dist, -dist, z_up),
                center_cam + vec3(dist, dist, z_up),
                center_cam + vec3(-dist, dist, z_up),
            ];
            let points = &v;
            let center = center_cam;

            let light_view = look_to_rh(center + dir, -dir, vec3(0.0, 0.0, 1.0));

            let mut near: f32 = f32::INFINITY;
            let mut far: f32 = f32::NEG_INFINITY;
            let mut left: f32 = f32::INFINITY;
            let mut right: f32 = f32::NEG_INFINITY;
            let mut bottom: f32 = f32::INFINITY;
            let mut top: f32 = f32::NEG_INFINITY;

            for &p in points {
                let p = vec3(p.x, p.y, p.z);
                let p = light_view * p.w(1.0);
                near = near.min(p.z);
                far = far.max(p.z);
                left = left.min(p.x);
                right = right.max(p.x);
                bottom = bottom.min(p.y);
                top = top.max(p.y);
            }

            let proj: Matrix4 = Ortho {
                left,
                right,
                bottom,
                top,
                near: near + near.signum() * 300.0,
                far,
            }
            .into();
            let projview = proj * light_view;
            let projview = texelsnap(resolution, projview);

            cascades.push(opengl_to_wgpu_matrix() * projview)
        }

        cascades
    }
}

pub fn texelsnap(resolution: f32, projview: Matrix4) -> Matrix4 {
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
    rounding * projview
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
        let c2 = [0.0            , 0.0, 0.0,  -1.0];
        let c3 = [0.0            , 0.0, self.near,   0.0];

        Matrix4::from([c0, c1, c2, c3])
    }
}
