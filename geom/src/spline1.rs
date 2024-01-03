use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Spline1 {
    pub from: f32,
    pub to: f32,
    pub from_derivative: f32,
    pub to_derivative: f32,
}

impl Default for Spline1 {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 0.0,
            from_derivative: 0.0,
            to_derivative: 0.0,
        }
    }
}

impl Spline1 {
    #[inline]
    pub fn get(&self, t: f32) -> f32 {
        (1.0 - t).powi(3) * self.from
            + 3.0 * t * (1.0 - t).powi(2) * (self.from + self.from_derivative)
            + 3.0 * t.powi(2) * (1.0 - t) * (self.to - self.to_derivative)
            + t.powi(3) * self.to
    }

    #[inline]
    pub fn derivative(&self, t: f32) -> f32 {
        -3.0 * (t - 1.0).powi(2) * self.from
            + 3.0 * (t - 1.0) * (3.0 * t - 1.0) * (self.from + self.from_derivative)
            + 3.0 * t * (2.0 - 3.0 * t) * (self.to - self.to_derivative)
            + 3.0 * t.powi(2) * self.to
    }

    #[inline]
    pub fn derivative_2(&self, t: f32) -> f32 {
        6.0 * (1.0 - t) * self.from
            + (18.0 * t - 12.0) * (self.from + self.from_derivative)
            + (6.0 - 18.0 * t) * (self.to - self.to_derivative)
            + 6.0 * t * self.to
    }

    #[allow(non_snake_case)]
    pub fn split_at(&self, t: f32) -> (Spline1, Spline1) {
        // https://upload.wikimedia.org/wikipedia/commons/1/11/Bezier_rec.png
        let mid = self.get(t);
        let H = (self.to - self.to_derivative) * t + (self.from + self.from_derivative) * (1.0 - t);

        let L2 = self.from + self.from_derivative * t;
        let L3 = L2 + (H - L2) * t;

        let from_spline = Spline1 {
            from: self.from,
            to: mid,
            from_derivative: L2 - self.from,
            to_derivative: mid - L3,
        };

        let R3 = self.to - self.to_derivative * (1.0 - t);
        let R2 = R3 + (H - R3) * (1.0 - t);

        let to_spline = Spline1 {
            from: mid,
            to: self.to,
            from_derivative: R2 - mid,
            to_derivative: self.to - R3,
        };

        (from_spline, to_spline)
    }

    pub fn smart_points(
        &self,
        detail: f32,
        start: f32,
        end: f32,
    ) -> impl Iterator<Item = f32> + '_ {
        self.smart_points_t(detail, start, end)
            .map(move |t| self.get(t))
    }

    pub fn smart_points_t(
        &self,
        detail: f32,
        start: f32,
        end: f32,
    ) -> impl Iterator<Item = f32> + '_ {
        let detail = detail.abs();

        std::iter::once(start)
            .chain(SmartPoints1 {
                spline: self,
                t: start,
                end,
                detail,
            })
            .chain(std::iter::once(end))
    }

    pub fn into_smart_points_t(
        self,
        detail: f32,
        start: f32,
        end: f32,
    ) -> impl Iterator<Item = f32> {
        let detail = detail.abs();

        std::iter::once(start)
            .chain(OwnedSmartPoints1 {
                spline: self,
                t: start,
                end,
                detail,
            })
            .chain(std::iter::once(end))
    }

    pub fn into_smart_points(self, detail: f32, start: f32, end: f32) -> impl Iterator<Item = f32> {
        self.into_smart_points_t(detail, start, end)
            .map(move |t| self.get(t))
    }

    pub fn points(&self, n: usize) -> impl Iterator<Item = f32> + '_ {
        (0..n).map(move |i| {
            let c = i as f32 / (n - 1) as f32;

            self.get(c)
        })
    }

    #[inline]
    fn step(&self, t: f32, detail: f32) -> f32 {
        let dot = self.derivative(t).abs();
        (detail / dot).min(0.15)
    }
}

pub struct SmartPoints1<'a> {
    spline: &'a Spline1,
    t: f32,
    end: f32,
    detail: f32,
}

impl<'a> Iterator for SmartPoints1<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.t += self.spline.step(self.t, self.detail);
        if self.t > self.end {
            return None;
        }
        Some(self.t)
    }
}

pub struct OwnedSmartPoints1 {
    spline: Spline1,
    t: f32,
    end: f32,
    detail: f32,
}

impl Iterator for OwnedSmartPoints1 {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.t += self.spline.step(self.t, self.detail);
        if self.t > self.end {
            return None;
        }
        Some(self.t)
    }
}
