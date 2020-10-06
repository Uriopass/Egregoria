use serde::{Deserialize, Serialize};

pub const SECONDS_PER_HOUR: i32 = 100;
pub const HOURS_PER_DAY: i32 = 24;
pub const SECONDS_PER_DAY: i32 = SECONDS_PER_HOUR * HOURS_PER_DAY;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct GameTime {
    /// Precise time in seconds elapsed since the start of the game
    pub timestamp: f64,

    /// Time elapsed since the last frame
    pub delta: f32,

    /// Time in seconds elapsed since the start of the game
    pub seconds: u32,

    /// Information about the time of the current day
    pub daytime: DayTime,
}

#[derive(Clone, Copy, PartialOrd, Ord, Eq, PartialEq, Serialize, Deserialize)]
pub struct DayTime {
    /// Days elapsed since the start of the game
    pub day: i32,

    /// Hours elapsed since the start of the day
    pub hour: i32,

    /// Seconds elapsed since the start of the hour
    pub second: i32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
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

pub struct RecTimeInterval {
    pub start_hour: i32,
    pub start_second: i32,

    pub end_hour: i32,
    pub end_second: i32,

    /// Does the interval go through midnight
    overlap: bool,
}

impl RecTimeInterval {
    pub fn new((start_hour, start_second): (i32, i32), (end_hour, end_second): (i32, i32)) -> Self {
        RecTimeInterval {
            start_hour,
            start_second,
            end_hour,
            end_second,

            overlap: end_hour < start_hour || (end_hour == start_hour && end_second < start_second),
        }
    }

    pub fn dist_until(&self, t: DayTime) -> i32 {
        let mut start_dt = DayTime {
            day: t.day,
            hour: self.start_hour,
            second: self.end_hour,
        };

        let end_dt = DayTime {
            day: t.day,
            hour: self.start_hour,
            second: self.end_hour,
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
            day: seconds / SECONDS_PER_DAY,
            hour: (seconds % SECONDS_PER_DAY) / SECONDS_PER_HOUR,
            second: (seconds % SECONDS_PER_HOUR),
        }
    }

    /// Returns the absolute difference (going either backward or forward in time) in seconds to the given daytime
    pub fn dist(&self, to: &DayTime) -> i32 {
        (self.gamesec() - to.gamesec()).abs()
    }

    /// Returns the number of seconds elapsed since the start of the day
    pub fn daysec(&self) -> i32 {
        self.hour * SECONDS_PER_HOUR + self.second
    }

    pub fn gamesec(&self) -> i32 {
        self.day * SECONDS_PER_DAY + self.daysec()
    }
}

impl GameTime {
    pub fn new(delta: f32, timestamp: f64) -> GameTime {
        if timestamp > 1e9 {
            log::warn!("Time went too fast, approaching limit.");
        }

        let seconds = timestamp as u32;
        GameTime {
            timestamp,
            delta,
            seconds,
            daytime: DayTime::new((seconds % SECONDS_PER_DAY as u32) as i32),
        }
    }

    /// Returns true every freq seconds
    pub fn tick(&self, freq: u32) -> bool {
        let time_near = (self.seconds / freq * freq + freq) as f64;
        self.timestamp > time_near && (self.timestamp - self.delta as f64) <= time_near
    }
}
