use crate::{vec2, vec4, Shape3, Vec2, Vec4, AABB3};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

pub type V3 = Vec3;

#[derive(Copy, Clone, Default, PartialEq)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Debug for Vec3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("V3(")?;
        Debug::fmt(&self.x, f)?;
        f.write_str(", ")?;
        Debug::fmt(&self.y, f)?;
        f.write_str(", ")?;
        Debug::fmt(&self.z, f)?;
        f.write_str(")")
    }
}

impl Shape3 for Vec3 {
    fn bbox(&self) -> AABB3 {
        AABB3::new(*self, *self)
    }
}

impl Serialize for Vec3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.x, self.y, self.z).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Vec3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (x, y, z) = Deserialize::deserialize(deserializer)?;
        Ok(Vec3 { x, y, z })
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[repr(C)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Eq for Vec3 {}

impl Display for Vec3 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("V3(")?;
        Display::fmt(&self.x, f)?;
        f.write_str(", ")?;
        Display::fmt(&self.y, f)?;
        f.write_str(", ")?;
        Display::fmt(&self.z, f)?;
        f.write_str(")")
    }
}

impl Display for Vec3d {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("V3d(")?;
        Display::fmt(&self.x, f)?;
        f.write_str(", ")?;
        Display::fmt(&self.y, f)?;
        f.write_str(", ")?;
        Display::fmt(&self.z, f)?;
        f.write_str(")")
    }
}

#[inline]
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

impl Add for Vec3 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add for &Vec3 {
    type Output = Vec3;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<Vec3> for &Vec3 {
    type Output = Vec3;

    #[inline]
    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<&Vec3> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn add(self, rhs: &Vec3) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vec3 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl SubAssign for Vec3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Sub for &Vec3 {
    type Output = Vec3;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn mul(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl MulAssign for Vec3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Div<Vec3> for f32 {
    type Output = Vec3;

    #[inline]
    fn div(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self / rhs.x,
            y: self / rhs.y,
            z: self / rhs.z,
        }
    }
}

impl Div<f32> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Div<Vec3> for Vec3 {
    type Output = Vec3;

    #[inline]
    fn div(self, rhs: Vec3) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::iter::Sum for Vec3 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut z = Vec3::ZERO;
        for x in iter {
            z += x;
        }
        z
    }
}

impl<'a> std::iter::Sum<&'a Vec3> for Vec3 {
    fn sum<I: Iterator<Item = &'a Vec3>>(iter: I) -> Self {
        let mut z = Vec3::ZERO;
        for &x in iter {
            z += x;
        }
        z
    }
}

impl DivAssign for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<f32> for Vec3 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl From<Vec3> for [f32; 3] {
    #[inline]
    fn from(v: Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

impl From<&Vec3> for [f32; 3] {
    #[inline]
    fn from(v: &Vec3) -> Self {
        [v.x, v.y, v.z]
    }
}

impl From<[f32; 3]> for Vec3 {
    #[inline]
    fn from(v: [f32; 3]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    #[inline]
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self { x, y, z }
    }
}

impl Vec3 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn x(x: f32) -> Self {
        Self { x, y: 0.0, z: 0.0 }
    }

    #[inline]
    pub const fn y(y: f32) -> Self {
        Self { x: 0.0, y, z: 0.0 }
    }

    #[inline]
    pub const fn z(z: f32) -> Self {
        Self { x: 0.0, y: 0.0, z }
    }

    #[inline]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    #[inline]
    pub fn lerp(self, other: Vec3, c: f32) -> Self {
        self * (1.0 - c) + other * c
    }

    #[inline]
    pub fn smoothstep(self, other: Vec3, t: f32) -> Self {
        self.lerp(other, t * t * (3.0 - t * 2.0))
    }

    #[inline]
    pub fn rotate_z(self, cossin: Vec2) -> Self {
        let xy = vec2(self.x, self.y);
        let xy = xy.rotated_by(cossin);
        vec3(xy.x, xy.y, self.z)
    }

    #[inline]
    pub fn z0(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: 0.0,
        }
    }

    #[inline]
    pub fn cross(self, other: Vec3) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// dir is the rotation such as X is sent to it and Z doesn't move
    #[inline]
    pub fn rotate_up(self, dir: Vec3) -> Self {
        let y = dir.cross(Vec3::Z).try_normalize().unwrap_or(Vec3::Y);
        let z = dir.cross(y).try_normalize().unwrap_or(Vec3::Z);
        vec3(
            self.x * dir.x + self.y * y.x + self.z * z.x,
            self.x * dir.y + self.y * y.y + self.z * z.y,
            self.x * dir.z + self.y * y.z + self.z * z.z,
        )
    }

    #[inline]
    pub const fn w(self, w: f32) -> Vec4 {
        vec4(self.x, self.y, self.z, w)
    }

    #[inline]
    pub fn mag(self) -> f32 {
        self.mag2().sqrt()
    }

    #[inline]
    pub fn mag2(self) -> f32 {
        self.dot(self)
    }

    pub fn from_angle(ang: f32, z: f32) -> Self {
        Self {
            x: ang.cos(),
            y: ang.sin(),
            z,
        }
    }

    #[inline]
    pub fn abs(self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    #[inline]
    pub fn xy(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    #[inline]
    pub fn up(mut self, z: f32) -> Self {
        self.z += z;
        self
    }

    #[inline]
    pub fn modulo(self, v: f32) -> Self {
        Self {
            x: self.x % v,
            y: self.y % v,
            z: self.z % v,
        }
    }

    #[inline]
    pub fn perp_up(self) -> Self {
        self.cross(Vec3::Z)
    }

    #[inline]
    pub fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor(),
            z: self.z.floor(),
        }
    }

    #[inline]
    pub fn fract(self) -> Self {
        Self {
            x: self.x.fract(),
            y: self.y.fract(),
            z: self.z.fract(),
        }
    }

    #[inline]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    #[inline]
    pub fn distance2(self, rhs: Self) -> f32 {
        (self - rhs).mag2()
    }

    #[inline]
    pub fn distance(self, rhs: Self) -> f32 {
        (self - rhs).mag()
    }

    #[inline]
    /// True if the distance to self is less than dist
    pub fn is_close(self, rhs: Self, dist: f32) -> bool {
        self.distance2(rhs) < dist * dist
    }

    #[inline]
    /// True if the distance to self is less than dist, but dist can be negative
    pub fn is_close_signed(self, rhs: Self, dist: f32) -> bool {
        if dist <= 0.0 {
            return false;
        }
        self.distance2(rhs) < dist * dist
    }

    #[inline]
    pub fn try_normalize(self) -> Option<Vec3> {
        let m = self.mag();
        if m > 0.0 {
            Some(self / m)
        } else {
            None
        }
    }

    #[inline]
    pub fn normalize(self) -> Vec3 {
        let m = self.mag();
        self / m
    }

    #[inline]
    pub fn try_normalize_to(self, v: f32) -> Option<Vec3> {
        let m = self.mag();
        if m > 0.0 {
            Some(self * (v / m))
        } else {
            None
        }
    }

    #[inline]
    pub fn normalize_to(self, v: f32) -> Vec3 {
        let m = self.mag();
        self * (v / m)
    }

    #[inline]
    pub fn dir_dist(self) -> Option<(Vec3, f32)> {
        let m = self.mag();
        if m > 0.0 {
            Some((self / m, m))
        } else {
            None
        }
    }

    #[inline]
    pub fn min(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x.min(other.x),
            y: self.y.min(other.y),
            z: self.z.min(other.z),
        }
    }

    #[inline]
    pub fn max(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x.max(other.x),
            y: self.y.max(other.y),
            z: self.z.max(other.z),
        }
    }

    #[inline]
    pub fn cap_magnitude(self, max: f32) -> Vec3 {
        let m = self.mag();
        if m > max {
            self * max / m
        } else {
            self
        }
    }

    #[inline]
    pub fn approx_eq(self, other: Vec3) -> bool {
        let m = self.distance2(other);
        m < f32::EPSILON
    }
}

//impl<'a> FromLua<'a> for Vec3 {
//    fn from_lua(value: Value<'a>, _: &'a mlua::Lua) -> mlua::Result<Self> {
//        let t = match value {
//            Value::Vector(v) => {
//                return Ok(Self {
//                    x: v.x(),
//                    y: v.y(),
//                    z: v.z(),
//                })
//            }
//            Value::Table(t) => t,
//            _ => {
//                return Err(mlua::Error::FromLuaConversionError {
//                    from: value.type_name(),
//                    to: "Vec3",
//                    message: Some("expected a table or vector".to_string()),
//                })
//            }
//        };
//        if let Ok(x) = t.get(1) {
//            return Ok(Self {
//                x,
//                y: t.get(2)?,
//                z: t.get(3)?,
//            });
//        }
//
//        let x = t.get("x")?;
//        let y = t.get("y")?;
//        let z = t.get("z")?;
//        Ok(Self { x, y, z })
//    }
//}
