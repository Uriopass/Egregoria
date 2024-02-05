use super::{Vec2, Vec2d};
use crate::polygon::Polygon;
use crate::{Circle, Intersect, Line, Lined, Shape, AABB};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub src: Vec2,
    pub dst: Vec2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Segmentd {
    pub src: Vec2d,
    pub dst: Vec2d,
}

impl Segment {
    pub fn new(src: Vec2, dst: Vec2) -> Self {
        Self { src, dst }
    }

    pub fn project(&self, p: Vec2) -> Vec2 {
        let diff: Vec2 = self.dst - self.src;
        let diff2: Vec2 = p - self.src;
        let diff3: Vec2 = p - self.dst;

        let proj1 = diff2.dot(diff);
        let proj2 = -diff3.dot(diff);

        if proj1 <= 0.0 {
            self.src
        } else if proj2 <= 0.0 {
            self.dst
        } else {
            let lol = proj1 / diff.mag2();
            self.src + diff * lol
        }
    }

    pub fn project_t(&self, p: Vec2) -> f32 {
        let diff = self.dst - self.src;
        let diff2 = p - self.src;
        let diff3 = p - self.dst;

        let proj1 = diff2.dot(diff);
        let proj2 = -diff3.dot(diff);

        if proj1 <= 0.0 {
            0.0
        } else if proj2 <= 0.0 {
            1.0
        } else {
            proj1 / diff.mag2()
        }
    }

    #[rustfmt::skip]
    pub fn distance(&self, s: &Segment) -> f32 {
        if self.intersects(s) {
            return 0.0;
        }
                   s.project(self.src).distance2(self.src)
        .min(s.project(self.dst).distance2(self.dst))
        .min(self.project(s.src).distance2(s.src))
        .min(self.project(s.dst).distance2(s.dst))
        .sqrt()
    }

    pub fn as_line(&self) -> Line {
        Line {
            src: self.src,
            dst: self.dst,
        }
    }

    pub fn center(&self) -> Vec2 {
        (self.src + self.dst) * 0.5
    }

    #[allow(clippy::manual_range_contains)]
    pub fn intersection_point(&self, other: &Segment) -> Option<Vec2> {
        // see https://stackoverflow.com/a/565282
        let r = self.vec();
        let s = other.vec();

        let r_cross_s = Vec2::cross(r, s);
        let q_minus_p = other.src - self.src;

        if r_cross_s != 0.0 {
            let t = Vec2::cross(q_minus_p, s / r_cross_s);
            let u = Vec2::cross(q_minus_p, r / r_cross_s);

            if 0.0 <= t && t <= 1.0 && 0.0 <= u && u <= 1.0 {
                return Some(self.src + r * t);
            }
        }
        None
    }

    pub fn resize(&mut self, length: f32) -> &mut Self {
        if let Some(v) = self.vec().try_normalize_to(length) {
            let mid = (self.src + self.dst) * 0.5;
            self.src = mid - v * 0.5;
            self.dst = mid + v * 0.5;
        }
        self
    }

    pub fn scale(&mut self, scale: f32) -> &mut Self {
        self.resize(self.vec().mag() * scale)
    }

    pub fn vec(&self) -> Vec2 {
        self.dst - self.src
    }

    pub fn to_polygon(self) -> Polygon {
        Polygon(vec![self.src, self.dst])
    }

    pub fn middle(&self) -> Vec2 {
        (self.src + self.dst) * 0.5
    }
}

impl Segmentd {
    pub fn new(src: Vec2d, dst: Vec2d) -> Self {
        Self { src, dst }
    }

    pub fn project(&self, p: Vec2d) -> Vec2d {
        let diff: Vec2d = self.dst - self.src;
        let diff2: Vec2d = p - self.src;
        let diff3: Vec2d = p - self.dst;

        let proj1 = diff2.dot(diff);
        let proj2 = -diff3.dot(diff);

        if proj1 <= 0.0 {
            self.src
        } else if proj2 <= 0.0 {
            self.dst
        } else {
            let lol = proj1 / diff.magnitude2();
            self.src + diff * lol
        }
    }

    pub fn project_t(&self, p: Vec2d) -> f64 {
        let diff = self.dst - self.src;
        let diff2 = p - self.src;
        let diff3 = p - self.dst;

        let proj1 = diff2.dot(diff);
        let proj2 = -diff3.dot(diff);

        if proj1 <= 0.0 {
            0.0
        } else if proj2 <= 0.0 {
            1.0
        } else {
            proj1 / diff.magnitude2()
        }
    }

    pub fn as_line(&self) -> Lined {
        Lined {
            src: self.src,
            dst: self.dst,
        }
    }

    #[allow(clippy::manual_range_contains)]
    pub fn intersection_point(&self, other: &Self) -> Option<Vec2d> {
        // see https://stackoverflow.com/a/565282
        let r = self.vec();
        let s = other.vec();

        let r_cross_s = Vec2d::cross(r, s);
        let q_minus_p = other.src - self.src;

        if r_cross_s != 0.0 {
            let t = Vec2d::cross(q_minus_p, s / r_cross_s);
            let u = Vec2d::cross(q_minus_p, r / r_cross_s);

            if 0.0 <= t && t <= 1.0 && 0.0 <= u && u <= 1.0 {
                return Some(self.src + r * t);
            }
        }
        None
    }

    pub fn resize(&mut self, length: f64) -> &mut Self {
        if let Some(v) = self.vec().try_normalize_to(length) {
            let mid = (self.src + self.dst) * 0.5;
            self.src = mid - v * 0.5;
            self.dst = mid + v * 0.5;
        }
        self
    }

    pub fn scale(&mut self, scale: f64) -> &mut Self {
        self.resize(self.vec().magnitude() * scale)
    }

    #[inline]
    pub fn vec(&self) -> Vec2d {
        self.dst - self.src
    }

    #[inline]
    pub fn middle(&self) -> Vec2d {
        (self.src + self.dst) * 0.5
    }
}

impl Shape for Segment {
    #[inline]
    fn bbox(&self) -> AABB {
        AABB::new_ll_ur(self.src.min(self.dst), self.src.max(self.dst))
    }
}

fn ccw(a: Vec2, b: Vec2, c: Vec2) -> bool {
    (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

impl Intersect<Segment> for Segment {
    #[inline]
    fn intersects(&self, s: &Segment) -> bool {
        ccw(self.src, s.src, s.dst) != ccw(self.dst, s.src, s.dst)
            && ccw(self.src, self.dst, s.src) != ccw(self.src, self.dst, s.dst)
    }
}

defer_inter!(Segment => Circle);
defer_inter!(Segment => AABB);

impl Intersect<Vec2> for Segment {
    #[inline]
    fn intersects(&self, _p: &Vec2) -> bool {
        false
    }
}
