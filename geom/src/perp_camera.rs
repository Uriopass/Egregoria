#![allow(dead_code)]
use crate::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub pos: Vec2,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub viewport_w: f32,
    pub viewport_h: f32,
    up: Vec3,
    aspect: f32,
    fovy: f32,
}

impl Camera {
    pub fn new(pos: Vec2, viewport_w: f32, viewport_h: f32) -> Self {
        Self {
            pos,
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: std::f32::consts::FRAC_PI_4,
            dist: 5000.0,
            viewport_w,
            viewport_h,
            up: (0.0, 0.0, 1.0).into(),
            aspect: viewport_w / viewport_h,
            fovy: 60.0,
        }
    }

    pub fn znear(height: f32) -> f32 {
        (height * 0.1).min(10.0)
    }

    pub fn offset(&self) -> Vec3 {
        let v = Vec2::from_angle(self.yaw);
        let horiz = self.pitch.cos();
        let vert = self.pitch.sin();
        (v * horiz).z(vert) * self.dist
    }

    pub fn set_viewport(&mut self, w: f32, h: f32) {
        self.viewport_w = w;
        self.viewport_h = h;
        self.aspect = w / h;
    }

    pub fn eye(&self) -> Vec3 {
        self.pos.z(0.0) + self.offset()
    }

    pub fn build_view_projection_matrix(&self) -> (Mat4, Mat4) {
        let eye = self.eye();
        let znear = Self::znear(eye.z);
        let zfar = znear * 4000.0;
        let view = look_at_rh(eye, self.pos.z(0.0), self.up);
        let proj = PerspectiveFov::new(
            self.fovy / 180.0 * std::f32::consts::PI,
            self.aspect,
            znear,
            zfar,
        )
        .mk_proj();

        let m = mul(opengl_to_wgpu_matrix(), mul(proj, view));
        (m, invert(&m).unwrap())
    }
}

#[rustfmt::skip]
pub fn opengl_to_wgpu_matrix() -> Mat4 {
    Mat4::from([
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    ])
}

pub fn look_at_rh(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_to_rh(eye, center - eye, up)
}

/// Create a homogeneous transformation matrix that will cause a vector to point at
/// `dir`, using `up` for orientation.
#[rustfmt::skip]
pub fn look_to_rh(eye: Vec3, dir: Vec3, up: Vec3) -> Mat4 {
    let f = dir.normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);

    Mat4::from([
        s.x,             u.x,              -f.x,           0.0,
        s.y,             u.y,              -f.y,           0.0,
        s.z,             u.z,              -f.z,           0.0,
        -eye.dot(s), -eye.dot(u), eye.dot(f), 1.0,
    ])
}
#[derive(Debug, Copy, Clone)]
pub struct PerspectiveFov {
    pub fovy_angle: f32, // Angle
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

type Mat4 = mint::ColumnMatrix4<f32>;
type Vec4 = mint::Vector4<f32>;

impl PerspectiveFov {
    pub fn new(fovy_angle: f32, aspect: f32, near: f32, far: f32) -> Self {
        PerspectiveFov {
            fovy_angle,
            aspect,
            near,
            far,
        }
    }
}

impl PerspectiveFov {
    pub fn mk_proj(&self) -> Mat4 {
        assert!(
            self.fovy_angle > 0.0,
            "The vertical field of view cannot be below zero, found: {:?}",
            self.fovy_angle
        );
        assert!(
            self.fovy_angle < 1.57,
            "The vertical field of view cannot be greater than a half turn, found: {:?}",
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
        assert!(
            self.far > 0.0,
            "The far plane distance cannot be below zero, found: {:?}",
            self.far
        );
        assert!(
            self.far > self.near,
            "The far plane cannot be closer than the near plane, found: far: {:?}, near: {:?}",
            self.far,
            self.near
        );

        let f = 1.0 / (self.fovy_angle / 2.0).tan();

        let c0r0 = f / self.aspect;
        let c0r1 = 0.0;
        let c0r2 = 0.0;
        let c0r3 = 0.0;

        let c1r0 = 0.0;
        let c1r1 = f;
        let c1r2 = 0.0;
        let c1r3 = 0.0;

        let c2r0 = 0.0;
        let c2r1 = 0.0;
        let c2r2 = (self.far + self.near) / (self.near - self.far);
        let c2r3 = -1.0;

        let c3r0 = 0.0;
        let c3r1 = 0.0;
        let c3r2 = (2.0 * self.far * self.near) / (self.near - self.far);
        let c3r3 = 0.0;

        mint::ColumnMatrix4::from([
            c0r0, c0r1, c0r2, c0r3, c1r0, c1r1, c1r2, c1r3, c2r0, c2r1, c2r2, c2r3, c3r0, c3r1,
            c3r2, c3r3,
        ])
    }
}

fn dot(a: &Vec4, b: &Vec4) -> f32 {
    (a.x * b.x + a.y * b.y) + (a.z * b.z + a.w * b.w)
}

fn invert(sel: &Mat4) -> Option<Mat4> {
    let tmp0 = unsafe { det_sub_proc_unsafe(sel, 1, 2, 3) };
    let det = dot(&tmp0, &Vec4::from([sel.x.x, sel.y.x, sel.z.x, sel.w.x]));
    if det.abs() < f32::EPSILON {
        None
    } else {
        let inv_det = 1.0 / det;
        let x = mul_scalar(tmp0, inv_det);
        let y = unsafe { mul_scalar(det_sub_proc_unsafe(sel, 0, 3, 2), inv_det) };
        let z = unsafe { mul_scalar(det_sub_proc_unsafe(sel, 0, 1, 3), inv_det) };
        let w = unsafe { mul_scalar(det_sub_proc_unsafe(sel, 0, 2, 1), inv_det) };
        Some(Mat4 { x, y, z, w })
    }
}

#[rustfmt::skip]
fn mul(lhs: Mat4, rhs: Mat4) -> Mat4 {
    let a = lhs.x;
    let b = lhs.y;
    let c = lhs.z;
    let d = lhs.w;

    Mat4::from([
        <Vec4 as Into<[f32; 4]>>::into(add(add(mul_scalar(a, rhs.x.x), mul_scalar(b, rhs.x.y)), add(mul_scalar(c, rhs.x.z), mul_scalar(d, rhs.x.w)))),
        <Vec4 as Into<[f32; 4]>>::into(add(add(mul_scalar(a, rhs.y.x), mul_scalar(b, rhs.y.y)), add(mul_scalar(c, rhs.y.z), mul_scalar(d, rhs.y.w)))),
        <Vec4 as Into<[f32; 4]>>::into(add(add(mul_scalar(a, rhs.z.x), mul_scalar(b, rhs.z.y)), add(mul_scalar(c, rhs.z.z), mul_scalar(d, rhs.z.w)))),
        <Vec4 as Into<[f32; 4]>>::into(add(add(mul_scalar(a, rhs.w.x), mul_scalar(b, rhs.w.y)), add(mul_scalar(c, rhs.w.z), mul_scalar(d, rhs.w.w)))),
    ])
}

#[rustfmt::skip]
pub fn mulmatvec(lhs: Mat4, rhs: Vec4) -> Vec4 {
    let a = lhs.x;
    let b = lhs.y;
    let c = lhs.z;
    let d = lhs.w;

    add(add(mul_scalar(a, rhs.x), mul_scalar(b, rhs.y)), add(mul_scalar(c, rhs.z), mul_scalar(d, rhs.w)))
}

unsafe fn det_sub_proc_unsafe(m: &Mat4, x: usize, y: usize, z: usize) -> mint::Vector4<f32> {
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
    let mut tmp = mul_elem(a, mul_elem(b, c));
    tmp = add(tmp, mul_elem(d, mul_elem(e, f)));
    tmp = add(tmp, mul_elem(g, mul_elem(h, i)));
    tmp = add(tmp, mul_scalar(mul_elem(a, mul_elem(e, i)), -1.0));
    tmp = add(tmp, mul_scalar(mul_elem(d, mul_elem(h, c)), -1.0));
    tmp = add(tmp, mul_scalar(mul_elem(g, mul_elem(b, f)), -1.0));
    tmp
}

fn add(a: Vec4, b: Vec4) -> Vec4 {
    Vec4::from([a.x + b.x, a.y + b.y, a.z + b.z, a.w + b.w])
}

fn mul_elem(a: Vec4, b: Vec4) -> Vec4 {
    Vec4::from([a.x * b.x, a.y * b.y, a.z * b.z, a.w * b.w])
}

fn mul_scalar(a: Vec4, b: f32) -> Vec4 {
    Vec4::from([a.x * b, a.y * b, a.z * b, a.w * b])
}
