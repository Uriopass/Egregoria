macro_rules! vec2 {
    ($a: expr, $b: expr) => {
        crate::geometry::Vec2::new($a, $b)
    };
    ($a: expr, $b: expr,) => {
        crate::geometry::Vec2::new($a, $b)
    };
}

pub mod intersections;
pub mod polyline;
pub mod rect;
pub mod segment;
pub mod splines;

pub mod vec2;
pub use vec2::*;

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
