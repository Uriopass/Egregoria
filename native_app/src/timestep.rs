use std::time::Instant;

pub struct Timestep {
    last_time: Instant,
    acc: f64,
    real_delta: f64,
}

impl Timestep {
    pub const DT: f64 = 1.0 / 50.0;
    const MAXTICKS: u32 = 10;

    pub fn new() -> Self {
        Self {
            last_time: Instant::now(),
            acc: 0.0,
            real_delta: 0.0,
        }
    }

    // Asks for how many ticks to simulate
    pub fn go_forward(&mut self, warp: f64) -> u32 {
        self.real_delta = self.last_time.elapsed().as_secs_f64();
        self.last_time = Instant::now();

        let mut ticks = 0;
        let advance = self.real_delta * warp as f64;
        if advance >= Self::MAXTICKS as f64 * Self::DT {
            self.real_delta = (Self::MAXTICKS as f64 * Self::DT) / warp;
            ticks = Self::MAXTICKS;
            self.acc = 0.0;
        } else {
            self.acc += advance;
            while self.acc >= Self::DT {
                self.acc -= Self::DT;
                ticks += 1;
            }
        }
        ticks
    }

    pub fn real_delta(&self) -> f32 {
        self.real_delta as f32
    }
}
