pub struct History {
    pub values: Vec<f32>,
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
        }
    }

    pub fn add_value(&mut self, value: f32) {
        self.values.rotate_left(1);
        *self.values.last_mut().unwrap() = value;
    }

    pub fn avg(&self) -> f32 {
        self.values.iter().sum::<f32>() / (self.values.len() as f32)
    }
}
