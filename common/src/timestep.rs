use std::time::{Duration, Instant};

const UP_DT: Duration = Duration::from_millis(20);

pub fn debug_up_dt() -> Duration {
    UP_DT
}

/// A timestep that can be used to update the game state.
/// It will try to keep a constant update rate.
/// Based on <https://gafferongames.com/post/fix_your_timestep/>
pub struct Timestep {
    last_time: Instant,
    acc: Duration,
    real_delta: Duration,
    pub period: Duration,
}

impl Default for Timestep {
    fn default() -> Self {
        Self::new(UP_DT)
    }
}

impl Timestep {
    const MAXTIME: Duration = Duration::from_millis(25);

    pub fn new(period: Duration) -> Self {
        Self {
            last_time: Instant::now(),
            acc: Default::default(),
            real_delta: Default::default(),
            period,
        }
    }

    pub fn prepare_frame(&mut self, warp: u32) {
        self.real_delta = self.last_time.elapsed();
        if self.real_delta > self.period * 3 {
            self.real_delta = self.period;
        }
        self.last_time = Instant::now();

        self.acc += self.real_delta * warp;
    }

    pub fn tick(&mut self) -> bool {
        if self.acc < self.period {
            return false;
        }
        if self.last_time.elapsed() > Timestep::MAXTIME {
            self.acc = Default::default();
            return true;
        }
        self.acc -= self.period;
        true
    }
}
