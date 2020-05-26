use super::Vec2;

pub struct Spline {
    pub from: Vec2,
    pub to: Vec2,
    pub from_derivative: Vec2,
    pub to_derivative: Vec2,
}

impl Default for Spline {
    fn default() -> Self {
        Self {
            from: [0.0, 0.0].into(),
            to: [0.0, 0.0].into(),
            from_derivative: [0.0, 0.0].into(),
            to_derivative: [0.0, 0.0].into(),
        }
    }
}

impl Spline {
    pub fn get(&self, t: f32) -> Vec2 {
        (1.0 - t).powi(3) * self.from
            + 3.0_f32 * t * (1.0 - t).powi(2) * (self.from + self.from_derivative)
            + 3.0_f32 * t.powi(2) * (1.0 - t) * (self.to - self.to_derivative)
            + t.powi(3) * self.to
    }
}
