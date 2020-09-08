mod circle;
mod intersections;
mod obb;
mod polygon;
mod polyline;
mod rect;
mod segment;
mod splines;
mod transform;
mod v2;

pub use circle::*;
pub use intersections::*;
pub use obb::*;
pub use polygon::*;
pub use polyline::*;
pub use rect::*;
pub use segment::*;
pub use splines::*;
pub use transform::*;
pub use v2::*;

pub fn minmax(x: &[Vec2]) -> Option<(Vec2, Vec2)> {
    let mut min: Vec2 = *x.get(0)?;
    let mut max: Vec2 = min;

    for &v in &x[1..] {
        min = min.min(v);
        max = max.max(v);
    }

    Some((min, max))
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

pub fn angle_lerp(src: Vec2, dst: Vec2, ang_amount: f32) -> Vec2 {
    let dot = src.dot(dst);
    let perp_dot = src.perp_dot(dst);
    if dot > 0.0 && perp_dot.abs() < ang_amount {
        return dst;
    }
    (src - src.perpendicular() * perp_dot.signum() * ang_amount).normalize()
}

pub fn linear_lerp(src: f32, dst: f32, amount: f32) -> f32 {
    src + (dst - src).min(amount).max(-amount)
}
