use cgmath::{InnerSpace, Vector2};

pub mod gridstore;
pub mod intersections;
pub mod polyline;
pub mod rect;
pub mod segment;
pub mod splines;

pub type Vec2 = Vector2<f32>;

macro_rules! vec2 {
    ($a: expr, $b: expr) => {
        crate::geometry::Vec2::new($a, $b)
    };
    ($a: expr, $b: expr,) => {
        crate::geometry::Vec2::new($a, $b)
    };
}

pub trait DirDist {
    fn dir_dist(&self) -> (Vec2, f32);
}

impl DirDist for Vec2 {
    fn dir_dist(&self) -> (Vec2, f32) {
        let m = self.magnitude();
        if m > 0.0 {
            (self / m, m)
        } else {
            (*self, m)
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
