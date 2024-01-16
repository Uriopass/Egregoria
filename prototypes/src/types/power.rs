use egui_inspect::debug_inspect_impl;
use mlua::{FromLua, Lua, Value};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
/// Power in watts (J/s)
pub struct Power(pub i64);
debug_inspect_impl!(Power);

impl Power {
    pub const ZERO: Power = Power(0);
    pub const MAX: Power = Power(i64::MAX);

    pub const fn new(watts: i64) -> Self {
        Self(watts)
    }

    pub const fn watts(&self) -> i64 {
        self.0
    }

    pub fn kilowatts(&self) -> f64 {
        self.0 as f64 / 1000.0
    }

    pub fn megawatts(&self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }

    pub fn gigawatts(&self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }
}

#[derive(Debug, Error)]
pub enum PowerParseError {
    #[error("Invalid unit: {0} (accepted: W, kW, MW, GW)")]
    InvalidUnit(String),
    #[error("Invalid number")]
    InvalidNumber,
    #[error("Power is too big")]
    TooBig,
}

impl FromStr for Power {
    type Err = PowerParseError;

    /// Parse a power value from a string. The unit can be W, kW or MW.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (mut number, rest) =
            common::parse_f64(s).map_err(|_| PowerParseError::InvalidNumber)?;

        let unit = rest.trim();

        match unit {
            "W" => {}
            "kW" => number *= 1000.0,
            "MW" => number *= 1000.0 * 1000.0,
            _ => return Err(PowerParseError::InvalidUnit(unit.to_string())),
        }

        if number > Power::MAX.0 as f64 {
            return Err(PowerParseError::TooBig);
        }

        Ok(Self(number as i64))
    }
}

impl Display for Power {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (unit, div) = match self.0 {
            0..=999 => ("W", 1.0),
            1000..=999_999 => ("kW", 1000.0),
            1_000_000..=999_999_999 => ("MW", 1_000_000.0),
            _ => ("GW", 1_000_000_000.0),
        };

        write!(f, "{:.2}{}", self.0 as f64 / div, unit)
    }
}

impl<'lua> FromLua<'lua> for Power {
    fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::Nil => Ok(Self(0)),
            Value::Integer(i) => Ok(Self(i as i64)),
            Value::Number(n) => {
                if n > i64::MAX as f64 {
                    return Err(mlua::Error::external(PowerParseError::TooBig));
                }
                Ok(Self(n as i64))
            }
            Value::String(s) => {
                let s = s.to_str()?.trim();
                if s.is_empty() {
                    return Ok(Self(0));
                }
                Self::from_str(s).map_err(mlua::Error::external)
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Power",
                message: Some("expected nil, string or number".into()),
            }),
        }
    }
}
