use std::time::{Duration, Instant};

pub struct Timestep {
    last_time: Instant,
    acc: f64,
    real_delta: f64,
}

impl Timestep {
    pub const DT: f64 = 1.0 / 50.0;
    const MAXTIME: Duration = Duration::from_millis(25);

    pub fn new() -> Self {
        Self {
            last_time: Instant::now(),
            acc: 0.0,
            real_delta: 0.0,
        }
    }

    // returns closure returning true if you have to go forward
    pub fn go_forward(&mut self, warp: f64, mut tick: impl FnMut()) {
        self.real_delta = self.last_time.elapsed().as_secs_f64();
        self.last_time = Instant::now();

        let advance = self.real_delta * warp as f64;
        self.acc += advance;

        while self.acc >= Self::DT {
            self.acc -= Self::DT;
            tick();
            if self.last_time.elapsed() > Self::MAXTIME {
                self.acc = 0.0;
                return;
            }
        }
    }

    pub fn real_delta(&self) -> f32 {
        self.real_delta as f32
    }
}
