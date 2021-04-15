#![allow(clippy::manual_range_contains)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::upper_case_acronyms)]

macro_rules! defer_inter {
    ($a:ty => $b:ty) => {
        impl Intersect<$b> for $a {
            fn intersects(&self, other: &$b) -> bool {
                other.intersects(self)
            }
        }
    };
}

mod aabb;
mod boldline;
mod camera;
mod circle;
mod color;
mod line;
mod obb;
mod polygon;
mod polyline;
mod ray;
mod segment;
pub mod skeleton;
mod splines;
mod transform;
mod v2;
mod v3;

pub use aabb::*;
pub use boldline::*;
pub use camera::*;
pub use circle::*;
pub use color::*;
pub use line::*;
pub use obb::*;
pub use polygon::*;
pub use polyline::*;
pub use ray::*;
pub use segment::*;
pub use splines::*;
pub use transform::*;
pub use v2::*;
pub use v3::*;

pub trait Intersect<T: Shape>: Shape {
    fn intersects(&self, shape: &T) -> bool;
}

pub trait Shape {
    fn bbox(&self) -> AABB;
}

pub enum ShapeEnum {
    OBB(OBB),
    Polygon(Polygon),
    Circle(Circle),
    AABB(AABB),
    Vec2(Vec2),
    BoldLine(BoldLine),
}

impl<T: Shape, U: Shape> Intersect<T> for &U
where
    U: Intersect<T>,
{
    fn intersects(&self, shape: &T) -> bool {
        U::intersects(self, shape)
    }
}

impl<T> Shape for &T
where
    T: Shape,
{
    fn bbox(&self) -> AABB {
        T::bbox(self)
    }
}

impl Shape for ShapeEnum {
    fn bbox(&self) -> AABB {
        match self {
            ShapeEnum::OBB(s) => s.bbox(),
            ShapeEnum::Polygon(s) => s.bbox(),
            ShapeEnum::Circle(s) => s.bbox(),
            ShapeEnum::AABB(s) => s.bbox(),
            ShapeEnum::Vec2(s) => s.bbox(),
            ShapeEnum::BoldLine(s) => s.bbox(),
        }
    }
}

impl Intersect<ShapeEnum> for ShapeEnum {
    fn intersects(&self, shape: &ShapeEnum) -> bool {
        match self {
            ShapeEnum::OBB(x) => x.intersects(shape),
            ShapeEnum::Polygon(x) => x.intersects(shape),
            ShapeEnum::Circle(x) => x.intersects(shape),
            ShapeEnum::AABB(x) => x.intersects(shape),
            ShapeEnum::Vec2(x) => x.intersects(shape),
            ShapeEnum::BoldLine(x) => x.intersects(shape),
        }
    }
}

macro_rules! impl_shape_enum {
    ($($t: ident),+) => {
        $(
            impl Intersect<$t> for ShapeEnum {
                fn intersects(&self, shape: &$t) -> bool {
                    match self {
                        ShapeEnum::OBB(x) => x.intersects(shape),
                        ShapeEnum::Polygon(x) => x.intersects(shape),
                        ShapeEnum::Circle(x) => x.intersects(shape),
                        ShapeEnum::AABB(x) => x.intersects(shape),
                        ShapeEnum::Vec2(x) => x.intersects(shape),
                        ShapeEnum::BoldLine(x) => x.intersects(shape),
                    }
                }
            }

            impl Intersect<ShapeEnum> for $t {
                fn intersects(&self, shape: &ShapeEnum) -> bool {
                    match shape {
                        ShapeEnum::OBB(x) => self.intersects(x),
                        ShapeEnum::Polygon(x) => self.intersects(x),
                        ShapeEnum::Circle(x) => self.intersects(x),
                        ShapeEnum::AABB(x) => self.intersects(x),
                        ShapeEnum::Vec2(x) => self.intersects(x),
                        ShapeEnum::BoldLine(x) => self.intersects(x),
                    }
                }
            }

            impl From<$t> for ShapeEnum {
                fn from(v: $t) -> Self {
                    Self::$t(v)
                }
            }
        )+
    }
}

impl_shape_enum!(OBB, Polygon, Vec2, Circle, AABB, BoldLine);

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

pub fn abs_lerp(src: f32, dst: f32, amount: f32) -> f32 {
    src + (dst - src).min(amount).max(-amount)
}

pub fn lerp(src: f32, dst: f32, coeff: f32) -> f32 {
    let coeff = coeff.max(0.0).min(1.0);
    src * (1.0 - coeff) + dst * coeff
}
