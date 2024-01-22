/// History is a circular buffer of values that can be used to calculate an average
/// of the last N values.
#[derive(Clone)]
pub struct History {
    pub values: Vec<f32>,
    pub start_value: u8,
}

impl Default for History {
    fn default() -> Self {
        Self::new(10)
    }
}

impl History {
    pub fn new(size: usize) -> Self {
        Self {
            values: vec![0.0; size],
            start_value: 0,
        }
    }

    pub fn add_value(&mut self, value: f32) {
        self.values.rotate_left(1);
        *self.values.last_mut().unwrap() = value;
        self.start_value = (self.start_value + 1).min(self.values.len() as u8);
    }

    pub fn avg(&self) -> f32 {
        self.values.iter().sum::<f32>() / (self.start_value as f32)
    }
}
