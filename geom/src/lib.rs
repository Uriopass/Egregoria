extern crate core;
macro_rules! defer_inter {
    ($a:ty => $b:ty) => {
        impl Intersect<$b> for $a {
            #[inline]
            fn intersects(&self, other: &$b) -> bool {
                other.intersects(self)
            }
        }
    };
}

mod aabb;
mod aabb3;
mod angle;
mod boldline;
mod boldspline;
mod circle;
mod color;
mod line;
mod line3;
mod matrix4;
mod obb;
mod perp_camera;
mod plane;
mod polygon;
mod polyline;
mod polyline3;
mod polyline3queue;
mod quaternion;
mod ray;
mod ray3;
mod segment;
mod segment3;
pub mod skeleton;
mod spline3;
mod splines;
mod transform;
mod v2;
mod v3;
mod v4;

pub use aabb::*;
pub use aabb3::*;
pub use angle::*;
pub use boldline::*;
pub use boldspline::*;
pub use circle::*;
pub use color::*;
pub use line::*;
pub use line3::*;
pub use matrix4::*;
pub use obb::*;
pub use perp_camera::*;
pub use plane::*;
pub use polygon::*;
pub use polyline::*;
pub use polyline3::*;
pub use polyline3queue::*;
pub use quaternion::*;
pub use ray::*;
pub use ray3::*;
pub use segment::*;
pub use segment3::*;
pub use spline3::*;
pub use splines::*;
pub use transform::*;
pub use transform::*;
pub use v2::*;
pub use v3::*;
pub use v4::*;

#[macro_export]
macro_rules! assert_delta {
    ($x:expr, $y:expr, $d:expr) => {
        assert!(
            $x - $y < $d || $y - $x < $d,
            "assert_delta failed: |{} - {}| < {}",
            $x,
            $y,
            $d
        );
    };
}

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
    BoldSpline(BoldSpline),
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
            ShapeEnum::BoldSpline(s) => s.bbox(),
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
            ShapeEnum::BoldSpline(x) => x.intersects(shape),
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
                        ShapeEnum::BoldSpline(x) => x.intersects(shape),
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
                        ShapeEnum::BoldSpline(x) => x.intersects(x),
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

impl_shape_enum!(OBB, Polygon, Vec2, Circle, AABB, BoldLine, BoldSpline);

pub fn minmax(x: impl IntoIterator<Item = Vec2>) -> Option<(Vec2, Vec2)> {
    let mut x = x.into_iter();
    let mut min: Vec2 = x.next()?;
    let mut max: Vec2 = min;

    for v in x {
        min = min.min(v);
        max = max.max(v);
    }

    Some((min, max))
}

pub fn minmax3(x: impl IntoIterator<Item = Vec3>) -> Option<(Vec3, Vec3)> {
    let mut x = x.into_iter();
    let mut min: Vec3 = x.next()?;
    let mut max: Vec3 = min;

    for v in x {
        min = min.min(v);
        max = max.max(v);
    }

    Some((min, max))
}

#[inline]
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

#[inline]
pub fn angle_lerp(src: Vec2, dst: Vec2, ang_amount: f32) -> Vec2 {
    let src_perp = src.perpendicular();
    let dot = src.dot(dst);
    let perp_dot = src_perp.dot(dst);
    if dot > 0.0 && perp_dot.abs() < ang_amount {
        return dst;
    }
    (src + src_perp * perp_dot.signum() * ang_amount).normalize()
}

#[inline]
pub fn angle_lerpxy(src: Vec3, dst: Vec3, ang_amount: f32) -> Vec3 {
    let m = dst.xy().magnitude();
    let lerped = angle_lerp(src.xy() / m, dst.xy() / m, ang_amount);
    (lerped * m).z(dst.z)
}

#[inline]
pub fn abs_lerp(src: f32, dst: f32, amount: f32) -> f32 {
    src + (dst - src).min(amount).max(-amount)
}

#[inline]
pub fn lerp(src: f32, dst: f32, coeff: f32) -> f32 {
    let coeff = coeff.max(0.0).min(1.0);
    src * (1.0 - coeff) + dst * coeff
}

impl flat_spatial::Vec2 for Vec2 {
    #[inline]
    fn x(&self) -> f32 {
        self.x
    }

    #[inline]
    fn y(&self) -> f32 {
        self.y
    }
}

impl flat_spatial::AABB for AABB {
    type V2 = Vec2;

    #[inline]
    fn ll(&self) -> Self::V2 {
        self.ll
    }

    #[inline]
    fn ur(&self) -> Self::V2 {
        self.ur
    }

    #[inline]
    fn intersects(&self, b: &Self) -> bool {
        <AABB as Intersect<AABB>>::intersects(self, b)
    }
}
