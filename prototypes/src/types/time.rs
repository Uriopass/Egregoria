use egui_inspect::{debug_inspect_impl, Inspect};
use mlua::{FromLua, Lua, Number, Value};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;
use thiserror::Error;

pub const SECONDS_PER_REALTIME_SECOND: u32 = 10;
pub const SECONDS_PER_HOUR: i32 = 60 * SECONDS_PER_MINUTE;
pub const SECONDS_PER_MINUTE: i32 = 60;
pub const MINUTES_PER_HOUR: i32 = 60;
pub const HOURS_PER_DAY: i32 = 24;
pub const SECONDS_PER_DAY: i32 = SECONDS_PER_HOUR * HOURS_PER_DAY;
pub const TICKS_PER_REALTIME_SECOND: u64 = 50;
pub const TICKS_PER_SECOND: u64 = TICKS_PER_REALTIME_SECOND / SECONDS_PER_REALTIME_SECOND as u64;
pub const TICKS_PER_MINUTE: u64 = TICKS_PER_SECOND * SECONDS_PER_MINUTE as u64;
pub const TICKS_PER_HOUR: u64 = TICKS_PER_SECOND * SECONDS_PER_HOUR as u64;
pub const DELTA_F64: f64 = 1.0 / TICKS_PER_REALTIME_SECOND as f64;
pub const DELTA: f32 = DELTA_F64 as f32;

/// The amount of time the game was updated
/// Used as a resource
#[derive(Default, PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct Tick(pub u64);
debug_inspect_impl!(Tick);

/// An in-game instant used to measure time differences
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Serialize, Deserialize)]
pub struct GameInstant(pub Tick);
debug_inspect_impl!(GameInstant);

/// The duration of a game event, in ticks
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Serialize, Deserialize)]
pub struct GameDuration(pub Tick);

debug_inspect_impl!(GameDuration);

/// The resource to know everything about the current in-game time
/// `GameTime` is subject to timewarp
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GameTime {
    /// The number of ticks elapsed since the start of the game
    pub tick: Tick,

    /// Monotonic time in (game) seconds elapsed since the start of the game.
    pub timestamp: f64,

    /// Game time in seconds elapsed since the start of the game
    pub seconds: u32,

    /// Information about the time of the current day
    pub daytime: DayTime,
}

/// A useful format to define intervals or points in game time
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DayTime {
    /// Days elapsed since the start of the game
    pub day: i32,

    /// Hours elapsed since the start of the day
    pub hour: i32,

    /// Minutes elapsed since the start of the hour
    pub minute: i32,

    /// Seconds elapsed since the start of the minute
    pub second: i32,
}

/// An interval of in-game time
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct TimeInterval {
    pub start: DayTime,
    pub end: DayTime,
}

impl TimeInterval {
    pub fn new(start: DayTime, end: DayTime) -> Self {
        TimeInterval { start, end }
    }

    pub fn dist(&self, t: DayTime) -> i32 {
        if t < self.start {
            self.start.gamesec() - t.gamesec()
        } else if t > self.end {
            t.gamesec() - self.end.gamesec()
        } else {
            0
        }
    }
}

/// A periodic interval of in-game time. Used for schedules. (for example 9am -> 6pm)
#[derive(Inspect, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct RecTimeInterval {
    pub start_hour: i32,
    pub start_minute: i32,

    pub end_hour: i32,
    pub end_minute: i32,

    /// Does the interval go through midnight
    overlap: bool,
}

impl RecTimeInterval {
    pub fn new((start_hour, start_minute): (i32, i32), (end_hour, end_minute): (i32, i32)) -> Self {
        RecTimeInterval {
            start_hour,
            start_minute,
            end_hour,
            end_minute,

            overlap: end_hour < start_hour || (end_hour == start_hour && end_minute < start_minute),
        }
    }

    pub fn dist_until(&self, t: DayTime) -> i32 {
        let mut start_dt = DayTime {
            day: t.day,
            hour: self.start_hour,
            minute: self.start_minute,
            second: 0,
        };

        let end_dt = DayTime {
            day: t.day,
            hour: self.end_hour,
            minute: self.end_minute,
            second: 0,
        };

        if !self.overlap {
            if t < start_dt {
                start_dt.gamesec() - t.gamesec()
            } else if t > end_dt {
                start_dt.day += 1;
                start_dt.gamesec() - t.gamesec()
            } else {
                0
            }
        } else if t >= end_dt && t <= start_dt {
            start_dt.gamesec() - t.gamesec()
        } else {
            0
        }
    }
}

impl DayTime {
    pub fn new(seconds: i32) -> DayTime {
        DayTime {
            day: 1 + seconds / SECONDS_PER_DAY,
            hour: (seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR,
            minute: (seconds % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE,
            second: seconds % SECONDS_PER_MINUTE,
        }
    }

    /// Returns the absolute difference (going either backward or forward in time) in seconds to the given daytime
    pub fn dist(&self, to: &DayTime) -> i32 {
        (self.gamesec() - to.gamesec()).abs()
    }

    /// Returns the number of seconds elapsed since the start of the day
    pub fn daysec(&self) -> i32 {
        self.hour * SECONDS_PER_HOUR + self.minute * SECONDS_PER_MINUTE + self.second
    }

    pub fn gamesec(&self) -> i32 {
        self.day * SECONDS_PER_DAY + self.daysec()
    }
}

impl GameTime {
    pub const HOUR: i32 = SECONDS_PER_HOUR;
    pub const DAY: i32 = SECONDS_PER_DAY;

    pub fn new(tick: Tick) -> GameTime {
        let timestamp = (tick.0 as f64 + 8.0 * TICKS_PER_HOUR as f64) / TICKS_PER_SECOND as f64;
        let seconds = timestamp as u32;

        GameTime {
            tick,
            timestamp,
            seconds,
            daytime: DayTime::new(seconds as i32),
        }
    }

    pub fn instant(&self) -> GameInstant {
        GameInstant(self.tick)
    }

    pub fn daysec(&self) -> f64 {
        self.timestamp % Self::DAY as f64
    }
}

impl GameDuration {
    pub fn from_secs(secs: u64) -> Self {
        GameDuration(Tick(secs * TICKS_PER_SECOND))
    }

    pub fn from_minutes(mins: u64) -> Self {
        GameDuration(Tick(mins * TICKS_PER_MINUTE))
    }

    pub fn seconds(&self) -> f64 {
        self.0 .0 as f64 / TICKS_PER_SECOND as f64
    }

    pub fn minutes(&self) -> f64 {
        self.0 .0 as f64 / TICKS_PER_MINUTE as f64
    }
}

impl GameInstant {
    /// Time elapsed since instant was taken
    pub fn elapsed(&self, time: &GameTime) -> GameDuration {
        GameDuration(Tick(time.tick.0 - self.0 .0))
    }
}

impl Display for GameInstant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let d = GameTime::new(self.0);
        write!(f, "{}", d.daytime)
    }
}

impl Display for DayTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}d {:02}:{:02}", self.day, self.hour, self.minute)
    }
}

impl Add<GameDuration> for GameInstant {
    type Output = GameInstant;

    fn add(self, rhs: GameDuration) -> Self::Output {
        GameInstant(Tick(self.0 .0 + rhs.0 .0))
    }
}

impl Add<GameInstant> for GameDuration {
    type Output = GameInstant;

    fn add(self, rhs: GameInstant) -> Self::Output {
        GameInstant(Tick(rhs.0 .0 + self.0 .0))
    }
}

impl Add<GameDuration> for GameTime {
    type Output = GameTime;

    fn add(self, rhs: GameDuration) -> Self::Output {
        GameTime::new(Tick(self.tick.0 + rhs.0 .0))
    }
}

impl Add<GameTime> for GameDuration {
    type Output = GameTime;

    fn add(self, rhs: GameTime) -> Self::Output {
        GameTime::new(Tick(rhs.tick.0 + self.0 .0))
    }
}

impl Sub<GameDuration> for GameInstant {
    type Output = GameInstant;

    fn sub(self, rhs: GameDuration) -> Self::Output {
        GameInstant(Tick(self.0 .0.saturating_sub(rhs.0 .0)))
    }
}

#[cfg(test)]
mod test {
    use common::timestep::debug_up_dt;

    #[test]
    fn assert_up_dt_ticks_per_second_match() {
        assert!((debug_up_dt().as_secs_f64() - super::DELTA_F64).abs() < 0.0001);
    }
}

impl Debug for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Tick {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for GameDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = self.0 .0 as f64;
        #[rustfmt::skip]
        let (v, unit) = match () {
            _ if x < TICKS_PER_MINUTE as f64 => (x / TICKS_PER_SECOND as f64, "s"),
            _ if x < TICKS_PER_HOUR   as f64 => (x / TICKS_PER_MINUTE as f64, "m"),
            _                                => (x / TICKS_PER_HOUR   as f64, "h"),
        };

        if (v.round() - v).abs() < 0.01 {
            write!(f, "{}{}", v.round(), unit)
        } else {
            write!(f, "{:.2}{}", v, unit)
        }
    }
}

#[derive(Debug, Error)]
pub enum TickParsingError {
    #[error("expected positive number")]
    NegativeNumber,
    #[error("expected valid number")]
    InvalidNumber,
    #[error("expected string ending with t, s, m, h or d (or tick, second, month, hour, day)")]
    InvalidSuffix,
}

impl FromStr for GameDuration {
    type Err = TickParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Tick::from_str(s).map(GameDuration)
    }
}

impl<'lua> FromLua<'lua> for GameDuration {
    fn from_lua(value: Value<'lua>, lua: &'lua Lua) -> mlua::Result<Self> {
        let result = Tick::from_lua(value, lua);
        match result {
            Ok(tick) => Ok(GameDuration(tick)),
            Err(mlua::Error::FromLuaConversionError {
                from,
                to: _,
                message,
            }) => Err(mlua::Error::FromLuaConversionError {
                from,
                to: "GameDuration",
                message,
            }),
            Err(e) => Err(e),
        }
    }
}

fn parse_tick_suffix(v: f64, suffix: &str) -> Option<f64> {
    Some(match suffix {
        "t" | "tick" | "ticks" => v,
        "s" | "sec" | "secs" | "second" | "seconds" => (v * TICKS_PER_SECOND as f64).round(),
        "m" | "min" | "mins" | "minute" | "minutes" => {
            (v * TICKS_PER_SECOND as f64 * SECONDS_PER_MINUTE as f64).round()
        }
        "h" | "hour" | "hours" => (v * TICKS_PER_SECOND as f64 * SECONDS_PER_HOUR as f64).round(),
        "d" | "day" | "days" => (v * TICKS_PER_SECOND as f64 * SECONDS_PER_DAY as f64).round(),
        _ => return None,
    })
}

impl FromStr for Tick {
    type Err = TickParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let (v, end) = common::parse_f64(s).map_err(|_| TickParsingError::InvalidNumber)?;
        let end = end.trim();
        let v = parse_tick_suffix(v, end).ok_or(TickParsingError::InvalidSuffix)?;
        if v < 0.0 {
            return Err(TickParsingError::NegativeNumber);
        }
        if v > u64::MAX as f64 {
            return Err(TickParsingError::InvalidNumber);
        }
        Ok(Tick(v as u64))
    }
}

impl<'lua> FromLua<'lua> for Tick {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        Ok(match value {
            Value::Integer(i) => {
                if i < 0 {
                    return Err(mlua::Error::FromLuaConversionError {
                        from: "negative integer",
                        to: "Tick",
                        message: Some("expected positive integer".into()),
                    });
                }
                Tick(i as u64)
            }
            Value::Number(n) => {
                if n < 0.0 {
                    return Err(mlua::Error::FromLuaConversionError {
                        from: "negative number",
                        to: "Tick",
                        message: Some("expected positive number".into()),
                    });
                }
                Tick(n.round() as u64)
            }
            Value::String(s) => {
                let s = s.to_str()?.trim();
                Tick::from_str(s).map_err(|e: TickParsingError| {
                    mlua::Error::FromLuaConversionError {
                        from: "string",
                        to: "Tick",
                        message: Some(e.to_string()),
                    }
                })?
            }
            Value::Table(t) => {
                let mut total = 0;
                for kv in t.pairs::<String, Number>() {
                    let (k, v) = kv?;

                    let v = parse_tick_suffix(v, &*k).ok_or_else(|| {
                        mlua::Error::FromLuaConversionError {
                            from: "string",
                            to: "Tick",
                            message: Some(TickParsingError::InvalidSuffix.to_string()),
                        }
                    })?;

                    if v < 0.0 {
                        return Err(mlua::Error::FromLuaConversionError {
                            from: "negative number",
                            to: "Tick",
                            message: Some("expected positive number".into()),
                        });
                    }
                    if total as f64 + v > u64::MAX as f64 {
                        return Err(mlua::Error::FromLuaConversionError {
                            from: "number",
                            to: "Tick",
                            message: Some("too big".into()),
                        });
                    }
                    total += v as u64;
                }
                Tick(total)
            }
            _ => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "Tick",
                    message: Some("expected number, table or string".into()),
                })
            }
        })
    }
}
