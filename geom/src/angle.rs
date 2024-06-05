use crate::Vec2;
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
#[serde(from = "f32", into = "f32")]
#[repr(transparent)]
pub struct Degrees(pub f32);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
#[serde(from = "f32", into = "f32")]
#[repr(transparent)]
pub struct Radians(pub f32);

impl Radians {
    pub const HALFPI: Self = Radians(std::f32::consts::FRAC_PI_2);
    pub const PI: Self = Radians(PI);
    pub const TAU: Self = Radians(TAU);
    pub const ZERO: Self = Radians(0.0);

    #[inline]
    pub fn vec2(self) -> Vec2 {
        Vec2 {
            x: self.0.cos(),
            y: self.0.sin(),
        }
    }

    #[inline]
    pub fn from_deg(deg: f32) -> Self {
        Self(deg * (PI / 180.0))
    }

    #[inline]
    pub fn normalize(&mut self) {
        self.0 %= TAU;
    }

    #[inline]
    pub fn cos(self) -> f32 {
        self.0.cos()
    }

    #[inline]
    pub fn sin(self) -> f32 {
        self.0.sin()
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    #[inline]
    pub fn to_degrees(self) -> Degrees {
        Degrees(self.0 * (180.0 / PI))
    }
}

impl Degrees {
    #[inline]
    pub fn vec2(self) -> Vec2 {
        Radians::from(self).vec2()
    }

    #[inline]
    pub fn normalize(&mut self) {
        self.0 %= 360.0;
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    #[inline]
    pub fn to_radians(self) -> Radians {
        Radians(self.0 * (PI / 180.0))
    }

    #[inline]
    pub fn from_rad(rad: f32) -> Self {
        Self(rad * (180.0 / PI))
    }
}

impl Sub for Degrees {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub for Radians {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Radians {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl SubAssign for Degrees {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Add for Degrees {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add for Radians {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Degrees {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign for Radians {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl From<Radians> for Degrees {
    #[inline]
    fn from(r: Radians) -> Self {
        Self(r.0 * (180.0 / PI))
    }
}

impl From<Degrees> for Radians {
    #[inline]
    fn from(r: Degrees) -> Self {
        Self(r.0 * (PI / 180.0))
    }
}

impl Mul<f32> for Radians {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Neg for Radians {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl From<f32> for Radians {
    #[inline]
    fn from(v: f32) -> Self {
        Self(v)
    }
}

impl From<f32> for Degrees {
    #[inline]
    fn from(v: f32) -> Self {
        Self(v)
    }
}

impl From<Degrees> for f32 {
    #[inline]
    fn from(d: Degrees) -> Self {
        d.0
    }
}

impl From<Radians> for f32 {
    #[inline]
    fn from(r: Radians) -> Self {
        r.0
    }
}
