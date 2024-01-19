use egui_inspect::debug_inspect_impl;
use mlua::{FromLua, Lua, Value};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Neg, SubAssign};
use std::str::FromStr;
use thiserror::Error;

/// Money in thousandths, can be negative when expressing debt.
#[derive(Default, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Money(pub i64);

debug_inspect_impl!(Money);

impl Money {
    pub const ZERO: Money = Money(0);
    pub const MAX: Money = Money(i64::MAX);

    pub fn from_float_bucks(v: f64) -> Self {
        Self((v * 10000.0) as i64)
    }

    pub fn from_float_cents(v: f64) -> Self {
        Self((v * 100.0) as i64)
    }

    pub const fn new_inner(inner: i64) -> Self {
        Self(inner)
    }

    pub const fn new_cents(cents: i64) -> Self {
        Self(cents * 100)
    }

    pub const fn new_bucks(base: i64) -> Self {
        Self(base * 10000)
    }

    pub fn inner(&self) -> i64 {
        self.0
    }

    pub fn cents(&self) -> i64 {
        self.0 / 100
    }

    pub fn bucks(&self) -> i64 {
        self.0 / 10000
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&(self.bucks()), f)?;
        let cent = (self.0 % 10000) / 100;
        if cent > 0 {
            f.write_str(".")?;
            if cent < 10 {
                f.write_str("0")?;
            }
            Display::fmt(&cent, f)?;
        }
        f.write_str("$")
    }
}

#[derive(Debug, Error)]
pub enum MoneyParseError {
    #[error("Invalid unit: {0} (accepted: $, c)")]
    InvalidUnit(String),
    #[error("Invalid number")]
    InvalidNumber,
    #[error("Money is too big")]
    TooBig,
}

impl FromStr for Money {
    type Err = MoneyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (mut number, rest) =
            common::parse_f64(s).map_err(|_| MoneyParseError::InvalidNumber)?;

        let unit = rest.trim();

        match unit {
            "$" | "" => {}
            "c" => number /= 100.0,
            _ => return Err(MoneyParseError::InvalidUnit(unit.to_string())),
        }

        if number * 10000.0 > i64::MAX as f64 {
            return Err(MoneyParseError::TooBig);
        }

        Ok(Money::from_float_bucks(number))
    }
}

impl<'lua> FromLua<'lua> for Money {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::Integer(i) => Ok(Money::new_bucks(i as i64)),
            Value::Number(n) => Ok(Money::from_float_bucks(n)),
            Value::String(s) => {
                let s = s.to_str()?.trim();
                if s.is_empty() {
                    return Ok(Money::ZERO);
                }
                s.parse().map_err(mlua::Error::external)
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Money",
                message: Some("expected a number or string".to_string()),
            }),
        }
    }
}

impl Mul<Money> for i64 {
    type Output = Money;

    fn mul(self, rhs: Money) -> Self::Output {
        Money(self * rhs.0)
    }
}

impl Neg for Money {
    type Output = Money;

    fn neg(self) -> Self::Output {
        Money(-self.0)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Self {
        iter.fold(Money::ZERO, |a, b| a + b)
    }
}

impl Debug for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

impl std::ops::Sub for Money {
    type Output = Money;

    fn sub(self, other: Money) -> Money {
        Money(self.0 - other.0)
    }
}

impl SubAssign for Money {
    fn sub_assign(&mut self, other: Money) {
        self.0 -= other.0;
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, other: Money) -> Money {
        Money(self.0 + other.0)
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, other: Money) {
        self.0 += other.0;
    }
}

impl Mul<i64> for Money {
    type Output = Money;

    fn mul(self, rhs: i64) -> Self::Output {
        Money(self.0 * rhs)
    }
}

impl Div<i64> for Money {
    type Output = Money;

    fn div(self, rhs: i64) -> Self::Output {
        Money(self.0 / rhs)
    }
}

impl Mul<f64> for Money {
    type Output = Money;

    fn mul(self, rhs: f64) -> Self::Output {
        Money((self.0 as f64 * rhs) as i64)
    }
}

impl Mul<Money> for f64 {
    type Output = Money;

    fn mul(self, rhs: Money) -> Self::Output {
        Money((rhs.0 as f64 * self) as i64)
    }
}
