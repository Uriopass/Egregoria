use cgmath::num_traits::Pow;
use cgmath::Vector2;

pub struct Spline {
    pub from: Vector2<f32>,
    pub to: Vector2<f32>,
    pub from_derivative: Vector2<f32>,
    pub to_derivative: Vector2<f32>,
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
    pub fn get(&self, t: f32) -> Vector2<f32> {
        (1.0 - t).pow(3) * self.from
            + 3.0_f32 * t * (1.0 - t).pow(2) * (self.from + self.from_derivative)
            + 3.0_f32 * t.pow(2) * (1.0 - t) * (self.to - self.to_derivative)
            + t.pow(3) * self.to
    }
}
