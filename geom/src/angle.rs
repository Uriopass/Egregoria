use crate::Vec2;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

#[derive(Serialize, Deserialize, Copy, Clone, PartialOrd, PartialEq, Default)]
#[serde(from = "f32", into = "f32")]
#[repr(transparent)]
pub struct Degrees(pub f32);

#[derive(Serialize, Deserialize, Copy, Clone, PartialOrd, PartialEq, Default)]
#[serde(from = "f32", into = "f32")]
#[repr(transparent)]
pub struct Radians(pub f32);

impl Radians {
    pub const HALFPI: Self = Radians(std::f32::consts::FRAC_PI_2);

    pub fn vec2(self) -> Vec2 {
        Vec2 {
            x: self.0.cos(),
            y: self.0.sin(),
        }
    }

    pub fn normalize(&mut self) {
        self.0 %= std::f32::consts::TAU;
    }

    pub fn cos(self) -> f32 {
        self.0.cos()
    }

    pub fn sin(self) -> f32 {
        self.0.sin()
    }

    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
}

impl Degrees {
    pub fn vec2(self) -> Vec2 {
        Radians::from(self).vec2()
    }

    pub fn normalize(&mut self) {
        self.0 %= 360.0;
    }

    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
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
    fn from(r: Radians) -> Self {
        Self(r.0 * (180.0 / PI))
    }
}

impl From<Degrees> for Radians {
    fn from(r: Degrees) -> Self {
        Self(r.0 * (PI / 180.0))
    }
}

impl Mul<f32> for Radians {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl From<f32> for Radians {
    fn from(v: f32) -> Self {
        Self(v)
    }
}

impl From<f32> for Degrees {
    fn from(v: f32) -> Self {
        Self(v)
    }
}

impl From<Degrees> for f32 {
    fn from(d: Degrees) -> Self {
        d.0
    }
}

impl From<Radians> for f32 {
    fn from(r: Radians) -> Self {
        r.0
    }
}
