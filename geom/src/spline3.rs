use super::Vec3;
use crate::{PolyLine3, Spline};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Spline3 {
    pub from: Vec3,
    pub to: Vec3,
    pub from_derivative: Vec3,
    pub to_derivative: Vec3,
}

impl Default for Spline3 {
    fn default() -> Self {
        Self {
            from: Vec3::ZERO,
            to: Vec3::ZERO,
            from_derivative: Vec3::ZERO,
            to_derivative: Vec3::ZERO,
        }
    }
}

impl Spline3 {
    pub fn get(&self, t: f32) -> Vec3 {
        (1.0 - t).powi(3) * self.from
            + 3.0 * t * (1.0 - t).powi(2) * (self.from + self.from_derivative)
            + 3.0 * t.powi(2) * (1.0 - t) * (self.to - self.to_derivative)
            + t.powi(3) * self.to
    }

    pub fn derivative(&self, t: f32) -> Vec3 {
        -3.0 * (t - 1.0).powi(2) * self.from
            + 3.0 * (t - 1.0) * (3.0 * t - 1.0) * (self.from + self.from_derivative)
            + 3.0 * t * (2.0 - 3.0 * t) * (self.to - self.to_derivative)
            + 3.0 * t.powi(2) * self.to
    }

    pub fn derivative_2(&self, t: f32) -> Vec3 {
        6.0 * (1.0 - t) * self.from
            + (18.0 * t - 12.0) * (self.from + self.from_derivative)
            + (6.0 - 18.0 * t) * (self.to - self.to_derivative)
            + 6.0 * t * self.to
    }

    pub fn flatten(&self) -> Spline {
        Spline {
            from: self.from.xy(),
            to: self.to.xy(),
            from_derivative: self.from_derivative.xy(),
            to_derivative: self.to_derivative.xy(),
        }
    }

    pub fn is_steep(&self, thickness: f32) -> bool {
        self.flatten().is_steep(thickness)
    }

    #[allow(non_snake_case)]
    pub fn split_at(&self, t: f32) -> (Spline3, Spline3) {
        // https://upload.wikimedia.org/wikipedia/commons/1/11/Bezier_rec.png
        let mid = self.get(t);
        let H = (self.to - self.to_derivative) * t + (self.from + self.from_derivative) * (1.0 - t);

        let L2 = self.from + self.from_derivative * t;
        let L3 = L2 + (H - L2) * t;

        let from_spline = Spline3 {
            from: self.from,
            to: mid,
            from_derivative: L2 - self.from,
            to_derivative: mid - L3,
        };

        let R3 = self.to - self.to_derivative * (1.0 - t);
        let R2 = R3 + (H - R3) * (1.0 - t);

        let to_spline = Spline3 {
            from: mid,
            to: self.to,
            from_derivative: R2 - mid,
            to_derivative: self.to - R3,
        };

        (from_spline, to_spline)
    }

    pub fn project_t(&self, p: Vec3, detail: f32) -> f32 {
        let mut le = self
            .smart_points_t(detail, 0.0, 1.0)
            .min_by_key(|&t| OrderedFloat(self.get(t).distance2(p)))
            .unwrap(); // Unwrap ok: smart_points always give start and end
        let mut ri = le + self.step(le, detail);
        let mut cur = (le + ri) * 0.5;

        let e = f32::EPSILON;

        while (ri - le) > e {
            cur = (ri + le) * 0.5;
            if self.get(cur - e).distance2(p) < self.get(cur + e).distance2(p) {
                ri = cur
            } else {
                le = cur
            }
        }

        cur
    }

    pub fn smart_points(
        &self,
        detail: f32,
        start: f32,
        end: f32,
    ) -> impl Iterator<Item = Vec3> + '_ {
        self.smart_points_t(detail, start, end)
            .map(move |t| self.get(t))
    }

    fn smart_points_t(&self, detail: f32, start: f32, end: f32) -> impl Iterator<Item = f32> + '_ {
        let detail = detail.abs();
        assert!(start >= 0.0);
        assert!(end <= 1.0);

        std::iter::once(start)
            .chain(SmartPoints3 {
                spline: self,
                t: start,
                end,
                detail,
            })
            .chain(std::iter::once(end))
    }

    pub fn points(&self, n: usize) -> impl Iterator<Item = Vec3> + '_ {
        (0..n).map(move |i| {
            let c = i as f32 / (n - 1) as f32;

            self.get(c)
        })
    }

    pub fn length(&self, detail: f32) -> f32 {
        PolyLine3::new(self.smart_points(detail, 0.0, 1.0).collect()).length()
    }

    fn step(&self, t: f32, detail: f32) -> f32 {
        let der = self.derivative(t);
        let dot = der
            .normalize()
            .perp_up()
            .dot(self.derivative_2(t))
            .abs()
            .sqrt();
        (detail / dot)
            .min(1.0 / (1.0 + 0.3 * der.z))
            .clamp(0.01, 0.1)
    }
}

pub struct SmartPoints3<'a> {
    spline: &'a Spline3,
    t: f32,
    end: f32,
    detail: f32,
}

impl<'a> Iterator for SmartPoints3<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.t += self.spline.step(self.t, self.detail);
        if self.t > self.end {
            return None;
        }
        Some(self.t)
    }
}
