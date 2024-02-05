use crate::{Circle, Intersect, Polygon, Radians, Shape, Vec3, Vec3d, AABB, OBB};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Copy, Clone, Default, PartialEq)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Serialize for Vec2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoded = (f32::to_bits(self.x) as u64) << 32 | f32::to_bits(self.y) as u64;
        serializer.serialize_u64(encoded)
    }
}

impl<'de> Deserialize<'de> for Vec2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = u64::deserialize(deserializer)?;
        let x = f32::from_bits(((encoded >> 32) & 0xFFFF_FFFF) as u32);
        let y = f32::from_bits((encoded & 0xFFFF_FFFF) as u32);
        Ok(Vec2 { x, y })
    }
}

#[derive(Copy, Clone, Default, PartialEq)]
#[repr(C)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

impl Debug for Vec2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("V2(")?;
        std::fmt::Display::fmt(&self.x, f)?;
        f.write_str(", ")?;
        std::fmt::Display::fmt(&self.y, f)?;
        f.write_str(")")
    }
}

impl Debug for Vec2d {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("V2d(")?;
        std::fmt::Display::fmt(&self.x, f)?;
        f.write_str(", ")?;
        std::fmt::Display::fmt(&self.y, f)?;
        f.write_str(")")
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Vec2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut x = self.x;
        if x == 0.0 {
            x = 0.0;
        }
        let mut y = self.y;
        if y == 0.0 {
            y = 0.0;
        }
        state.write_u32(x.to_bits());
        state.write_u32(y.to_bits());
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Vec2d {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut x = self.x;
        if x == 0.0 {
            x = 0.0;
        }
        let mut y = self.y;
        if y == 0.0 {
            y = 0.0;
        }
        state.write_u64(x.to_bits());
        state.write_u64(y.to_bits());
    }
}

#[inline]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

#[inline]
pub const fn vec2d(x: f64, y: f64) -> Vec2d {
    Vec2d { x, y }
}

impl Vec2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    pub const fn x(x: f32) -> Self {
        Self { x, y: 0.0 }
    }

    #[inline]
    pub const fn y(y: f32) -> Self {
        Self { x: 0.0, y }
    }

    #[inline]
    pub fn z(self, z: f32) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z,
        }
    }

    #[inline]
    pub const fn z0(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: 0.0,
        }
    }

    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
    pub const XY: Self = Self { x: 1.0, y: 1.0 };

    /// perpendicular return a vector rotated 90 degrees clockwise
    /// [1, 0] -> [0, -1]
    ///
    ///    .---->
    ///    |    )
    ///    |   /
    ///    v <'
    ///
    #[inline]
    pub fn perpendicular(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    pub fn try_bisect(self, other: Self) -> Option<Self> {
        let c = self.mag() * other + self * other.mag();
        c.try_normalize()
    }

    #[inline]
    pub fn to_arr(self) -> [f32; 2] {
        [self.x, self.y]
    }

    #[inline]
    pub fn mag(self) -> f32 {
        self.mag2().sqrt()
    }

    #[inline]
    pub fn mag2(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    #[inline]
    pub fn perp_dot(self, rhs: Self) -> f32 {
        self.dot(rhs.perpendicular())
    }

    pub fn is_close(self, other: Self, close_dist: f32) -> bool {
        self.distance2(other) <= close_dist * close_dist
    }

    #[inline]
    pub fn distance2(self, rhs: Self) -> f32 {
        (self - rhs).mag2()
    }

    #[inline]
    pub fn distance(self, rhs: Self) -> f32 {
        (self - rhs).mag()
    }

    /// Returns the angle between self and other in range [-pi; pi]
    #[inline]
    pub fn angle(self, other: Vec2) -> f32 {
        f32::atan2(Self::perp_dot(self, other), Self::dot(self, other))
    }

    /// Returns the angle in range [-pi; pi] such that if v is unitary
    /// v == (angle.cos(), angle.sin())
    #[inline]
    pub fn angle_cossin(self) -> Radians {
        Radians(f32::atan2(self.y, self.x))
    }

    #[inline]
    pub fn from_angle(angle: Radians) -> Vec2 {
        Self {
            x: angle.0.cos(),
            y: angle.0.sin(),
        }
    }

    #[inline]
    pub fn try_normalize(self) -> Option<Vec2> {
        let m = self.mag();
        if m > f32::EPSILON {
            Some(self / m)
        } else {
            None
        }
    }

    #[inline]
    pub fn flipy(self) -> Vec2 {
        Self {
            x: self.x,
            y: -self.y,
        }
    }

    #[inline]
    pub fn flipx(self) -> Vec2 {
        Self {
            x: -self.x,
            y: self.y,
        }
    }

    #[inline]
    pub fn normalize(self) -> Vec2 {
        let m = self.mag();
        self / m
    }

    #[inline]
    pub fn try_normalize_to(self, v: f32) -> Option<Vec2> {
        let m = self.mag();
        if m > 0.0 {
            Some(self * (v / m))
        } else {
            None
        }
    }

    #[inline]
    pub fn normalize_to(self, v: f32) -> Vec2 {
        let m = self.mag();
        self * (v / m)
    }

    #[inline]
    pub fn dir_dist(self) -> Option<(Vec2, f32)> {
        let m = self.mag();
        if m > 0.0 {
            Some((self / m, m))
        } else {
            None
        }
    }

    #[inline]
    pub fn lerp(self, other: Self, coeff: f32) -> Self {
        self * (1.0 - coeff) + other * coeff
    }

    #[inline]
    pub fn snap(self, x_period: f32, y_period: f32) -> Self {
        Self {
            x: (self.x / x_period).round() * x_period,
            y: (self.y / y_period).round() * y_period,
        }
    }

    #[inline]
    pub fn sign(self) -> Self {
        Self {
            x: self.x.signum(),
            y: self.y.signum(),
        }
    }

    #[inline]
    pub fn min(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    #[inline]
    pub fn max(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    #[inline]
    pub fn modulo(self, v: f32) -> Self {
        Self {
            x: self.x % v,
            y: self.y % v,
        }
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    #[inline]
    pub fn ceil(self) -> Self {
        Self {
            x: self.x.ceil(),
            y: self.y.ceil(),
        }
    }

    #[inline]
    pub fn fract(self) -> Self {
        Self {
            x: self.x.fract(),
            y: self.y.fract(),
        }
    }

    #[inline]
    pub fn round(self) -> Self {
        Self {
            x: self.x.round(),
            y: self.y.round(),
        }
    }

    #[inline]
    pub fn cap_magnitude(self, max: f32) -> Vec2 {
        let m = self.mag();
        if m > max {
            self * max / m
        } else {
            self
        }
    }

    #[inline]
    pub fn approx_eq(self, other: Vec2) -> bool {
        (self.x - other.x).abs() < 1e-6 && (self.y - other.y).abs() < 1e-6
    }

    #[inline]
    pub fn rotated_by(self, cossin: Vec2) -> Self {
        self.x * cossin - self.y * cossin.perpendicular()
    }

    #[inline]
    /// Rotates self by angle in radians counter-clockwise
    pub fn rotated_by_angle(self, angle: Radians) -> Self {
        self.rotated_by(vec2(angle.cos(), angle.sin()))
    }
}

impl Vec2d {
    #[inline]
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    #[inline]
    pub const fn splat(v: f64) -> Self {
        Self { x: v, y: v }
    }

    #[inline]
    pub const fn x(x: f64) -> Self {
        Self { x, y: 0.0 }
    }

    #[inline]
    pub const fn y(y: f64) -> Self {
        Self { x: 0.0, y }
    }

    #[inline]
    pub fn z(self, z: f64) -> Vec3d {
        Vec3d {
            x: self.x,
            y: self.y,
            z,
        }
    }

    #[inline]
    pub const fn z0(self) -> Vec3d {
        Vec3d {
            x: self.x,
            y: self.y,
            z: 0.0,
        }
    }

    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    #[inline]
    pub fn perpendicular(self) -> Self {
        Self {
            x: self.y,
            y: -self.x,
        }
    }

    #[inline]
    pub fn magnitude(self) -> f64 {
        self.magnitude2().sqrt()
    }

    #[inline]
    pub fn magnitude2(self) -> f64 {
        self.dot(self)
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> f64 {
        self.x * rhs.y - self.y * rhs.x
    }

    #[inline]
    pub fn perp_dot(self, rhs: Self) -> f64 {
        self.dot(rhs.perpendicular())
    }

    pub fn is_close(self, other: Self, close_dist: f64) -> bool {
        self.distance2(other) <= close_dist * close_dist
    }

    #[inline]
    pub fn distance2(self, rhs: Self) -> f64 {
        (self - rhs).magnitude2()
    }

    #[inline]
    pub fn distance(self, rhs: Self) -> f64 {
        (self - rhs).magnitude()
    }

    /// Returns the angle between self and other in range [-pi; pi]
    #[inline]
    pub fn angle(self, other: Self) -> f64 {
        f64::atan2(Self::perp_dot(self, other), Self::dot(self, other))
    }

    #[inline]
    pub fn from_angle(angle: f64) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    #[inline]
    pub fn try_normalize(self) -> Option<Self> {
        let m = self.magnitude();
        if m > f64::EPSILON {
            Some(self / m)
        } else {
            None
        }
    }

    #[inline]
    pub fn flipy(self) -> Self {
        Self {
            x: self.x,
            y: -self.y,
        }
    }

    #[inline]
    pub fn flipx(self) -> Self {
        Self {
            x: -self.x,
            y: self.y,
        }
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let m = self.magnitude();
        self / m
    }

    #[inline]
    pub fn try_normalize_to(self, v: f64) -> Option<Self> {
        let m = self.magnitude();
        if m > 0.0 {
            Some(self * (v / m))
        } else {
            None
        }
    }

    #[inline]
    pub fn normalize_to(self, v: f64) -> Self {
        let m = self.magnitude();
        self * (v / m)
    }

    #[inline]
    pub fn dir_dist(self) -> Option<(Self, f64)> {
        let m = self.magnitude();
        if m > 0.0 {
            Some((self / m, m))
        } else {
            None
        }
    }

    pub fn lerp(self, other: Self, coeff: f64) -> Self {
        self * (1.0 - coeff) + other * coeff
    }

    pub fn snap(self, x_period: f64, y_period: f64) -> Self {
        Self {
            x: (self.x / x_period).round() * x_period,
            y: (self.y / y_period).round() * y_period,
        }
    }

    pub fn sign(self) -> Self {
        Self {
            x: self.x.signum(),
            y: self.y.signum(),
        }
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
        }
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
        }
    }

    #[inline]
    pub fn modulo(self, v: f64) -> Self {
        Self {
            x: self.x % v,
            y: self.y % v,
        }
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
        }
    }

    #[inline]
    pub fn fract(self) -> Self {
        Self {
            x: self.x.fract(),
            y: self.y.fract(),
        }
    }

    #[inline]
    pub fn cap_magnitude(self, max: f64) -> Self {
        let m = self.magnitude();
        if m > max {
            self * max / m
        } else {
            self
        }
    }

    #[inline]
    pub fn approx_eq(self, other: Self) -> bool {
        (self.x - other.x).abs() < 1e-6 && (self.y - other.y).abs() < 1e-6
    }

    #[inline]
    pub fn rotated_by(self, cossin: Self) -> Self {
        self.x * cossin - self.y * cossin.perpendicular()
    }
}

impl Shape for Vec2 {
    #[inline]
    fn bbox(&self) -> AABB {
        AABB {
            ll: (*self),
            ur: (*self),
        }
    }
}

impl Intersect<AABB> for Vec2 {
    #[inline]
    fn intersects(&self, aabb: &AABB) -> bool {
        aabb.contains(*self)
    }
}

impl Intersect<OBB> for Vec2 {
    #[inline]
    fn intersects(&self, shape: &OBB) -> bool {
        shape.contains(*self)
    }
}

impl Intersect<Polygon> for Vec2 {
    #[inline]
    fn intersects(&self, shape: &Polygon) -> bool {
        shape.contains(*self)
    }
}

impl Intersect<Circle> for Vec2 {
    #[inline]
    fn intersects(&self, shape: &Circle) -> bool {
        shape.intersects(self)
    }
}

impl Intersect<Vec2> for Vec2 {
    #[inline]
    fn intersects(&self, shape: &Vec2) -> bool {
        self == shape
    }
}

impl Eq for Vec2 {}

impl Add for Vec2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &Vec2 {
    type Output = Vec2;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Vec2> for &Vec2 {
    type Output = Vec2;

    #[inline]
    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Vec2> for Vec2 {
    type Output = Vec2;

    #[inline]
    fn add(self, rhs: &Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub for &Vec2 {
    type Output = Vec2;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    #[inline]
    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = Vec2;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign<Vec2> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl MulAssign<f32> for Vec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs
    }
}

impl Div<Vec2> for f32 {
    type Output = Vec2;

    #[inline]
    fn div(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self / rhs.x,
            y: self / rhs.y,
        }
    }
}

impl Div<f32> for Vec2 {
    type Output = Vec2;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<Vec2> for Vec2 {
    type Output = Vec2;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::iter::Sum for Vec2 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut z = Vec2::ZERO;
        for x in iter {
            z += x;
        }
        z
    }
}

impl<'a> std::iter::Sum<&'a Vec2> for Vec2 {
    fn sum<I: Iterator<Item = &'a Vec2>>(iter: I) -> Self {
        let mut z = Vec2::ZERO;
        for &x in iter {
            z += x;
        }
        z
    }
}

impl DivAssign for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<f32> for Vec2 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl From<(f32, f32)> for Vec2 {
    #[inline]
    fn from(v: (f32, f32)) -> Self {
        Self { x: v.0, y: v.1 }
    }
}

impl From<Vec2> for (f32, f32) {
    #[inline]
    fn from(v: Vec2) -> Self {
        (v.x, v.y)
    }
}

impl From<Vec2> for [f32; 2] {
    #[inline]
    fn from(v: Vec2) -> Self {
        [v.x, v.y]
    }
}

impl From<[f32; 2]> for Vec2 {
    #[inline]
    fn from(v: [f32; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}

impl Eq for Vec2d {}

impl Add for Vec2d {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &Vec2d {
    type Output = Vec2d;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vec2d {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Vec2d> for &Vec2d {
    type Output = Vec2d;

    #[inline]
    fn add(self, rhs: Vec2d) -> Self::Output {
        Vec2d {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<&Vec2d> for Vec2d {
    type Output = Vec2d;

    #[inline]
    fn add(self, rhs: &Vec2d) -> Self::Output {
        Vec2d {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Vec2d {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vec2d {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub for &Vec2d {
    type Output = Vec2d;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2d {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl SubAssign for Vec2d {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<Vec2d> for f64 {
    type Output = Vec2d;

    #[inline]
    fn mul(self, rhs: Vec2d) -> Self::Output {
        Vec2d {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl Mul<f64> for Vec2d {
    type Output = Vec2d;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vec2d> for Vec2d {
    type Output = Vec2d;

    #[inline]
    fn mul(self, rhs: Vec2d) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl MulAssign<Vec2d> for Vec2d {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl MulAssign<f64> for Vec2d {
    #[inline]
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs
    }
}

impl Div<Vec2d> for f64 {
    type Output = Vec2d;

    #[inline]
    fn div(self, rhs: Vec2d) -> Self::Output {
        Vec2d {
            x: self / rhs.x,
            y: self / rhs.y,
        }
    }
}

impl Div<f64> for Vec2d {
    type Output = Vec2d;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl Div<Vec2d> for Vec2d {
    type Output = Vec2d;

    #[inline]
    fn div(self, rhs: Vec2d) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl Neg for Vec2d {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::iter::Sum for Vec2d {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut z = Vec2d::ZERO;
        for x in iter {
            z += x;
        }
        z
    }
}

impl<'a> std::iter::Sum<&'a Vec2d> for Vec2d {
    fn sum<I: Iterator<Item = &'a Vec2d>>(iter: I) -> Self {
        let mut z = Vec2d::ZERO;
        for &x in iter {
            z += x;
        }
        z
    }
}

impl DivAssign for Vec2d {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<f64> for Vec2d {
    #[inline]
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl From<Vec2d> for [f64; 2] {
    #[inline]
    fn from(v: Vec2d) -> Self {
        [v.x, v.y]
    }
}

impl From<[f64; 2]> for Vec2d {
    #[inline]
    fn from(v: [f64; 2]) -> Self {
        Self { x: v[0], y: v[1] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn test_rotate() {
        let v = vec2(1.0, 0.0);
        assert!(vec2(0.0, 1.0).approx_eq(v.rotated_by_angle(Radians(FRAC_PI_2))))
    }

    #[test]
    fn test_angle_cossin() {
        assert_eq!(vec2(1.0, 0.0).angle_cossin(), Radians::ZERO);
    }
}
