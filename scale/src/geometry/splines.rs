use super::Vec2;
use ordered_float::OrderedFloat;

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
            + 3.0 * t * (1.0 - t).powi(2) * (self.from + self.from_derivative)
            + 3.0 * t.powi(2) * (1.0 - t) * (self.to - self.to_derivative)
            + t.powi(3) * self.to
    }

    pub fn derivative(&self, t: f32) -> Vec2 {
        -3.0 * (t - 1.0).powi(2) * self.from
            + 3.0 * (t - 1.0) * (3.0 * t - 1.0) * (self.from + self.from_derivative)
            + 3.0 * t * (2.0 - 3.0 * t) * (self.to - self.to_derivative)
            + 3.0 * t.powi(2) * self.to
    }

    pub fn derivative_2(&self, t: f32) -> Vec2 {
        6.0 * (1.0 - t) * self.from
            + (18.0 * t - 12.0) * (self.from + self.from_derivative)
            + (6.0 - 18.0 * t) * (self.to - self.to_derivative)
            + 6.0 * t * self.to
    }

    pub fn smart_points(&self, detail: f32) -> impl Iterator<Item = Vec2> + '_ {
        let detail = detail.abs();
        let mut t = 0.0;
        let mut points = vec![];

        while t <= 1.0 {
            points.push(t);
            let dot = self
                .derivative(t)
                .normalize()
                .perp_dot(self.derivative_2(t))
                .abs()
                .sqrt();
            t += detail / dot.max(std::f32::EPSILON);
        }
        let mul;
        if points.len() == 1 {
            mul = 1.0;
            points.push(1.0);
        } else {
            mul = 1.0 / points.iter().max_by_key(|x| OrderedFloat(**x)).unwrap();
        }

        points.into_iter().map(move |t| self.get(t * mul))
    }

    pub fn points(&self, n: usize) -> impl Iterator<Item = Vec2> + '_ {
        (0..n).map(move |i| {
            let c = i as f32 / (n - 1) as f32;

            self.get(c)
        })
    }
}
