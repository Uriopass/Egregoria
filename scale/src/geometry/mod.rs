use cgmath::{InnerSpace, Vector2};

pub mod gridstore;
pub mod intersections;
pub mod polyline;
pub mod rect;
pub mod segment;
pub mod splines;

type Vec2 = Vector2<f32>;

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
