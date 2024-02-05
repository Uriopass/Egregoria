use crate::get_lua;
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

    /// Monotonic time in (game) seconds elapsed since the start of the game as a double.
    pub timestamp: f64,

    /// Game time in seconds elapsed since the start of the game
    pub seconds: u32,

    /// Information about the time of the current day
    pub daytime: DayTime,
}

/// DayTime is a time of the day in the game
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
/// The interval is inclusive on the start and exclusive on the end
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct TimeInterval {
    pub start_seconds: i32,
    pub end_seconds: i32,
}

impl TimeInterval {
    pub fn new(start: DayTime, end: DayTime) -> Self {
        TimeInterval {
            start_seconds: start.gamesec(),
            end_seconds: end.gamesec(),
        }
    }

    pub fn is_active(&self, t: DayTime) -> bool {
        let t_sec = t.gamesec();
        (self.start_seconds..self.end_seconds).contains(&t_sec)
    }

    pub fn dist(&self, t: DayTime) -> i32 {
        (t.gamesec() - self.start_seconds).max(0)
    }
}

/// A periodic interval of in-game time. Used for schedules. (for example 9am -> 6pm)
/// The interval is inclusive on the start and exclusive on the end
#[derive(Inspect, Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecTimeInterval {
    /// Start of the interval in seconds. Always less or equal than end_seconds.
    start_seconds: i32,
    /// End of the interval in seconds. Always greater or equal than start_seconds.
    end_seconds: i32,

    /// Is the interval inverted (for example 6pm -> 9am)
    /// Meaning start_seconds is actually the end of the interval
    inverted: bool,
}

impl RecTimeInterval {
    /// Creates a new interval from two hour/minute tuples
    pub fn new(start_hour_minute: (i32, i32), end_hour_minute: (i32, i32)) -> Self {
        Self::new_daysec(
            start_hour_minute.0 * SECONDS_PER_HOUR + start_hour_minute.1 * SECONDS_PER_MINUTE,
            end_hour_minute.0 * SECONDS_PER_HOUR + end_hour_minute.1 * SECONDS_PER_MINUTE,
        )
    }

    /// Creates a new interval from two daysec
    pub fn new_daysec(start_daysec: i32, end_daysec: i32) -> Self {
        let mut start_seconds = start_daysec;
        let mut end_seconds = end_daysec;

        let inverted = end_seconds < start_seconds;

        if inverted {
            std::mem::swap(&mut start_seconds, &mut end_seconds);
        }

        RecTimeInterval {
            start_seconds,
            end_seconds,

            inverted,
        }
    }

    /// 0/24 interval
    pub fn never() -> Self {
        RecTimeInterval {
            start_seconds: 0,
            end_seconds: 0,
            inverted: false,
        }
    }

    /// 24/24 interval
    pub fn always() -> Self {
        RecTimeInterval {
            start_seconds: 0,
            end_seconds: 0,
            inverted: true,
        }
    }

    /// Is the given time in the interval
    pub fn is_active(&self, t: &DayTime) -> bool {
        let t_day = t.daysec();

        self.inverted ^ (self.start_seconds..self.end_seconds).contains(&t_day)
    }

    /// Time until the next interval
    pub fn dist_start(&self, t: &DayTime) -> i32 {
        let t_day = t.daysec();

        if self.is_active(t) {
            return 0;
        }

        if self.inverted {
            return self.end_seconds - t_day;
        }

        let d = self.start_seconds - t_day;

        if t_day >= self.end_seconds {
            return d + SECONDS_PER_DAY;
        }
        d
    }
}

impl DayTime {
    /// Creates a new DayTime from the number of seconds elapsed since the start of the game
    pub fn new(seconds: i32) -> DayTime {
        DayTime {
            day: seconds / SECONDS_PER_DAY,
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
    #[inline]
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
        let timestamp = Self::DAY as f64
            + (tick.0 as f64 + 8.0 * TICKS_PER_HOUR as f64) / TICKS_PER_SECOND as f64;
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

    /// Returns the number of seconds elapsed since the start of the day
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

                    let v = parse_tick_suffix(v, &k).ok_or_else(|| {
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DayTimeParsingError {
    #[error("expected valid number")]
    InvalidNumber,
    #[error("expected h or : (e.g 1:30 or 1h30)")]
    InvalidHourSuffix,
    #[error("hour not in range 0-23")]
    HourNotInRange,
    #[error("minute not in range 0-59")]
    MinuteNotInRange,
}

/// Parse a string like "1h 30" or "1h30" or "1h 30" or "1h30" or "00h" or "00h 00" or "00:12"
/// Returns the parsed time and the rest of the string
fn parse_daytime(s: &str) -> Result<(DayTime, &str), DayTimeParsingError> {
    use DayTimeParsingError::*;
    let s = s.trim();
    let (hour_part, rest) = common::parse_f64(s).map_err(|_| InvalidNumber)?;
    let rest = rest.trim();
    if rest.is_empty() {
        return Err(InvalidHourSuffix);
    }
    let rest = match rest.split_once([':', 'h']) {
        Some((_, rest)) => rest,
        None => return Err(InvalidHourSuffix),
    };
    let mut rest = rest.trim();
    let minute_part;

    (minute_part, rest) = common::parse_f64(rest).unwrap_or((0.0, rest));

    let hour = hour_part as i32;
    let minute = minute_part as i32;

    if !(0..HOURS_PER_DAY).contains(&hour) {
        return Err(HourNotInRange);
    }
    if !(0..MINUTES_PER_HOUR).contains(&minute) {
        return Err(MinuteNotInRange);
    }

    Ok((
        DayTime {
            day: 0,
            hour,
            minute,
            second: 0,
        },
        rest,
    ))
}

impl<'lua> FromLua<'lua> for DayTime {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        let result = match value {
            Value::String(s) => {
                let s = s.to_str()?;
                let (dt, rest) =
                    parse_daytime(s).map_err(|e| mlua::Error::FromLuaConversionError {
                        from: "string",
                        to: "DayTime",
                        message: Some(e.to_string()),
                    })?;
                let rest = rest.trim();
                if !rest.is_empty() {
                    return Err(mlua::Error::FromLuaConversionError {
                        from: "string",
                        to: "DayTime",
                        message: Some(format!("unexpected suffix: {}", rest)),
                    });
                }
                dt
            }
            Value::Table(t) => DayTime {
                day: get_lua(&t, "day")?,
                hour: get_lua(&t, "hour")?,
                minute: get_lua(&t, "minute")?,
                second: get_lua(&t, "second")?,
            },
            _ => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: value.type_name(),
                    to: "DayTime",
                    message: Some("expected string or table".into()),
                })
            }
        };
        Ok(result)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RecTimeIntervalParsingError {
    #[error("invalid start of interval: {0}")]
    InvalidStart(DayTimeParsingError),
    #[error("invalid end of interval: {0}")]
    InvalidEnd(DayTimeParsingError),
    #[error("invalid separator, expected ->")]
    InvalidSeparator,
    #[error("unexpected suffix: {0}")]
    UnexpectedSuffix(String),
}

/// Parse a string like "1h30 -> 2h30" or "always" or "never"
/// Returns the parsed RecTimeInterval
///
fn parse_rectime(s: &str) -> Result<RecTimeInterval, RecTimeIntervalParsingError> {
    let s = s.trim();
    match s {
        "always" => return Ok(RecTimeInterval::always()),
        "never" => return Ok(RecTimeInterval::never()),
        _ => {}
    };
    let (start, rest) = parse_daytime(s).map_err(RecTimeIntervalParsingError::InvalidStart)?;
    let rest = rest.trim();
    let rest = match rest.split_once("->") {
        Some((_, rest)) => rest,
        None => return Err(RecTimeIntervalParsingError::InvalidSeparator),
    };
    let (end, rest) = parse_daytime(rest).map_err(RecTimeIntervalParsingError::InvalidEnd)?;
    let rest = rest.trim();
    if !rest.is_empty() {
        return Err(RecTimeIntervalParsingError::UnexpectedSuffix(
            rest.to_string(),
        ));
    }
    Ok(RecTimeInterval::new_daysec(start.daysec(), end.daysec()))
}

impl<'lua> FromLua<'lua> for RecTimeInterval {
    fn from_lua(value: Value<'lua>, _lua: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::String(s) => {
                let s = s.to_str()?;
                parse_rectime(s).map_err(|e| mlua::Error::FromLuaConversionError {
                    from: "string",
                    to: "RecTimeInterval",
                    message: Some(e.to_string()),
                })
            }
            Value::Table(t) => {
                let start: DayTime = get_lua(&t, "start")?;
                let end: DayTime = get_lua(&t, "end")?;
                Ok(RecTimeInterval::new_daysec(start.daysec(), end.daysec()))
            }
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "RecTimeInterval",
                message: Some("expected string or table".into()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rectime_interval() {
        fn h(hour: i32) -> DayTime {
            DayTime::new(hour * SECONDS_PER_HOUR)
        }
        use super::*;
        let interval = RecTimeInterval::new((8, 0), (18, 0));
        assert!(!interval.is_active(&h(0)));
        assert!(interval.is_active(&h(8)));
        assert!(interval.is_active(&h(17)));
        assert!(!interval.is_active(&h(18)));
        assert!(!interval.is_active(&h(7)));
        assert!(!interval.is_active(&h(19)));

        assert_eq!(interval.dist_start(&h(0)), 8 * SECONDS_PER_HOUR);
        assert_eq!(interval.dist_start(&h(8)), 0);
        assert_eq!(interval.dist_start(&h(17)), 0);
        assert_eq!(interval.dist_start(&h(18)), 14 * SECONDS_PER_HOUR);
        assert_eq!(interval.dist_start(&h(7)), 1 * SECONDS_PER_HOUR);

        let interval = RecTimeInterval::new((18, 0), (8, 0));
        assert!(interval.is_active(&h(0)));
        assert!(!interval.is_active(&h(8)));
        assert!(!interval.is_active(&h(17)));
        assert!(interval.is_active(&h(18)));
        assert!(interval.is_active(&h(7)));
        assert!(interval.is_active(&h(19)));

        assert_eq!(interval.dist_start(&h(0)), 0);
        assert_eq!(interval.dist_start(&h(8)), 10 * SECONDS_PER_HOUR);
        assert_eq!(interval.dist_start(&h(17)), 1 * SECONDS_PER_HOUR);
        assert_eq!(interval.dist_start(&h(18)), 0);
        assert_eq!(interval.dist_start(&h(7)), 0);
    }

    #[test]
    #[rustfmt::skip]
    fn test_daytime_parsing() {
        use super::*;
        use DayTimeParsingError::*;

        assert_eq!(parse_daytime("1h30"), Ok((DayTime::new(1 * SECONDS_PER_HOUR + 30 * SECONDS_PER_MINUTE), "")));
        assert_eq!(parse_daytime("1:30"), Ok((DayTime::new(1 * SECONDS_PER_HOUR + 30 * SECONDS_PER_MINUTE), "")));
        assert_eq!(parse_daytime("1h 30"), Ok((DayTime::new(1 * SECONDS_PER_HOUR + 30 * SECONDS_PER_MINUTE), "")));
        assert_eq!(parse_daytime("1h"), Ok((DayTime::new(1 * SECONDS_PER_HOUR), "")));
        assert_eq!(parse_daytime("1h 30"), Ok((DayTime::new(1 * SECONDS_PER_HOUR + 30 * SECONDS_PER_MINUTE), "")));
        assert_eq!(parse_daytime("1h30 -> ok"), Ok((DayTime::new(1 * SECONDS_PER_HOUR + 30 * SECONDS_PER_MINUTE), " -> ok")));
        assert_eq!(parse_daytime("aa"), Err(InvalidNumber));
        assert_eq!(parse_daytime("1o30"), Err(InvalidHourSuffix));
        assert_eq!(parse_daytime("-1h30"), Err(HourNotInRange));
        assert_eq!(parse_daytime("30h30"), Err(HourNotInRange));
        assert_eq!(parse_daytime("1h60"), Err(MinuteNotInRange));
        assert_eq!(parse_daytime("1h-10"), Err(MinuteNotInRange));
    }

    #[test]
    #[rustfmt::skip]
    fn rectime_parsing() {
        use super::*;
        use RecTimeIntervalParsingError::*;
        use DayTimeParsingError::*;

        assert_eq!(parse_rectime("1h30 -> 2h30"), Ok(RecTimeInterval::new((1, 30), (2, 30))));
        assert_eq!(parse_rectime("18h30->2h30"), Ok(RecTimeInterval::new((18, 30), (2, 30))));
        assert_eq!(parse_rectime("invalid -> 2h30"), Err(InvalidStart(InvalidNumber)));
        assert_eq!(parse_rectime("1h30 -> invalid"), Err(InvalidEnd(InvalidNumber)));
        assert_eq!(parse_rectime("1h30 -> 2h30 -> invalid"), Err(UnexpectedSuffix("-> invalid".into())));
    }
}
