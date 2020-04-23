use cgmath::{InnerSpace, Vector2};

macro_rules! vec2 {
    ($a: expr, $b: expr) => {
        crate::geometry::Vec2::new($a, $b)
    };
    ($a: expr, $b: expr,) => {
        crate::geometry::Vec2::new($a, $b)
    };
}

pub mod gridstore;
pub mod intersections;
pub mod polyline;
pub mod rect;
pub mod segment;
pub mod splines;

pub type Vec2 = Vector2<f32>;

pub trait Vec2Impl {
    fn dir_dist(&self) -> Option<(Vec2, f32)>;

    fn cap_magnitude(&self, max: f32) -> Vec2;
}

impl Vec2Impl for Vec2 {
    fn dir_dist(&self) -> Option<(Vec2, f32)> {
        let m = self.magnitude();
        if m > 0.0 {
            Some((self / m, m))
        } else {
            None
        }
    }

    fn cap_magnitude(&self, max: f32) -> Vec2 {
        let m = self.magnitude();
        if m > max {
            self * max / m
        } else {
            *self
        }
    }
}

pub fn pseudo_angle(v: Vec2) -> f32 {
    debug_assert!((v.magnitude2() - 1.0).abs() <= 1e-5);
    let dx = v.x;
    let dy = v.y;
    let p = dx / (dx.abs() + dy.abs());

    if dy < 0.0 {
        p - 1.0
    } else {
        1.0 - p
    }
}
