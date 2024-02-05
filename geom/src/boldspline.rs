use super::{Vec2, Vec3};
use crate::aabb::AABB;
use crate::{vec2, BoldLine, Circle, Intersect, Polygon, Segment, Shape, Spline, OBB};
use serde::{Deserialize, Serialize};

/// An ordered list of at least one point forming a broken line
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoldSpline {
    pub spline: Spline,
    pub radius: f32,
}

impl BoldSpline {
    pub fn new(line: Spline, radius: f32) -> Self {
        BoldSpline {
            spline: line,
            radius,
        }
    }
}

impl Shape for BoldSpline {
    fn bbox(&self) -> AABB {
        self.spline.bbox().expand(self.radius)
    }
}

defer_inter!(Vec2 => BoldSpline);
impl Intersect<Vec2> for BoldSpline {
    fn intersects(&self, p: &Vec2) -> bool {
        self.spline
            .get(self.spline.project_t(*p, 1.0))
            .is_close(*p, self.radius)
    }
}

defer_inter!(Circle => BoldSpline);
impl Intersect<Circle> for BoldSpline {
    fn intersects(&self, p: &Circle) -> bool {
        self.spline
            .get(self.spline.project_t(p.center, 1.0))
            .is_close(p.center, self.radius + p.radius)
    }
}

defer_inter!(Polygon => BoldSpline);
impl Intersect<Polygon> for BoldSpline {
    fn intersects(&self, p: &Polygon) -> bool {
        if p.is_empty() {
            return false;
        }
        if p.len() == 1 {
            return self.intersects(&p[0]);
        }

        let mut points = self.spline.smart_points(0.1, 0.0, 1.0).peekable();
        if let Some(&v) = points.peek() {
            if p.contains(v) {
                return true;
            }
        }

        let bbox = p.bbox().expand(self.radius);
        while let Some(v) = points.next() {
            let Some(&peek) = points.peek() else {
                return false;
            };

            let segment = Segment::new(v, peek);
            if !bbox.intersects(&segment) {
                continue;
            }

            for s in p.segments() {
                if segment.distance(&s) <= self.radius {
                    return true;
                }
            }
        }

        false
    }
}

pub static mut DEBUG_POS: Vec<Vec3> = Vec::new();
pub static mut DEBUG_OBBS: Vec<OBB> = Vec::new();
pub static mut DEBUG_SPLINES: Vec<Spline> = Vec::new();

defer_inter!(BoldLine => BoldSpline);
impl Intersect<BoldLine> for BoldSpline {
    fn intersects(&self, b: &BoldLine) -> bool {
        for segment in b.line.segments() {
            let v = segment.vec().normalize_to(b.radius).perpendicular();
            let obb = OBB::new_corners([
                segment.src - v,
                segment.src + v,
                segment.dst + v,
                segment.dst - v,
            ]);

            if obb.intersects(self) {
                return true;
            }
        }
        false
    }
}

impl Intersect<BoldSpline> for BoldSpline {
    fn intersects(&self, b: &BoldSpline) -> bool {
        fn intersect_inner(a: &Spline, b: &Spline) -> bool {
            let sbb = a.wide_bbox();
            let bbb = b.wide_bbox();
            if !sbb.intersects(&bbb) {
                return false;
            }
            if sbb.area() + bbb.area() < 0.1 {
                return true;
            }
            let (c, d) = a.split_at(0.5);
            let (e, f) = b.split_at(0.5);
            intersect_inner(&c, &e)
                || intersect_inner(&c, &f)
                || intersect_inner(&d, &e)
                || intersect_inner(&d, &f)
        }

        intersect_inner(&self.spline, &b.spline)
    }
}

defer_inter!(AABB => BoldSpline);
impl Intersect<AABB> for BoldSpline {
    fn intersects(&self, p: &AABB) -> bool {
        fn intersect_inner(a: &Spline, b: &AABB) -> bool {
            let sbb = a.wide_bbox();
            if !sbb.intersects(b) {
                return false;
            }
            if sbb.area() < 0.1 {
                return true;
            }
            let (c, d) = a.split_at(0.5);
            intersect_inner(&c, b) || intersect_inner(&d, b)
        }
        let big = p.expand(self.radius);
        intersect_inner(&self.spline, &big)
    }
}

defer_inter!(OBB => BoldSpline);
impl Intersect<OBB> for BoldSpline {
    fn intersects(&self, p: &OBB) -> bool {
        let mut s = self.spline;
        let [v1, v2] = p.axis();
        let w = v1.mag();
        let h = v2.mag();

        let rot = v1.flipy();

        s.from -= p.corners[0];
        s.from = s.from.rotated_by(rot);
        s.from_derivative = s.from_derivative.rotated_by(rot);
        s.to -= p.corners[0];
        s.to = s.to.rotated_by(rot);
        s.to_derivative = s.to_derivative.rotated_by(rot);

        let aabb = AABB::new_ll_ur(Vec2::ZERO, vec2(w * w, w * h));

        BoldSpline {
            spline: s,
            radius: self.radius,
        }
        .intersects(&aabb)
    }
}
