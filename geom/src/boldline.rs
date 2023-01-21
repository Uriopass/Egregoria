use super::Vec2;
use crate::aabb::AABB;
use crate::{Circle, Intersect, PolyLine, Polygon, Shape, OBB};
use serde::{Deserialize, Serialize};

/// An ordered list of at least one point forming a broken line
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoldLine {
    pub line: PolyLine,
    pub radius: f32,
}
impl BoldLine {
    pub fn new(line: PolyLine, radius: f32) -> Self {
        BoldLine { line, radius }
    }

    pub fn expand(&mut self, r: f32) {
        self.radius += r;
    }
}

impl Shape for BoldLine {
    #[inline]
    fn bbox(&self) -> AABB {
        self.line.bbox().expand(self.radius)
    }
}

defer_inter!(Vec2 => BoldLine);
impl Intersect<Vec2> for BoldLine {
    #[inline]
    fn intersects(&self, p: &Vec2) -> bool {
        self.line.project(*p).is_close(*p, self.radius)
    }
}

defer_inter!(Circle => BoldLine);
impl Intersect<Circle> for BoldLine {
    #[inline]
    fn intersects(&self, p: &Circle) -> bool {
        self.line
            .project(p.center)
            .is_close(p.center, self.radius + p.radius)
    }
}

defer_inter!(Polygon => BoldLine);
impl Intersect<Polygon> for BoldLine {
    fn intersects(&self, p: &Polygon) -> bool {
        if p.is_empty() {
            return false;
        }
        if p.len() == 1 {
            return self.intersects(&p[0]);
        }
        if p.contains(self.line.first()) {
            return true;
        }
        let pbox = p.bbox().expand(self.radius);
        for s in self.line.segments() {
            if !pbox.intersects(&s) {
                continue;
            }
            for s2 in p.segments() {
                if s.distance(&s2) <= self.radius {
                    return true;
                }
            }
        }
        false
    }
}

impl Intersect<BoldLine> for BoldLine {
    fn intersects(&self, b: &BoldLine) -> bool {
        let sbb = self.bbox();
        let bbb = b.bbox();
        let r = self.radius + b.radius;
        for seg1 in self.line.segments().filter(|x| x.intersects(&bbb)) {
            for seg2 in b.line.segments().filter(|x| x.intersects(&sbb)) {
                if seg1.intersects(&seg2) {
                    return true;
                }
                if seg1.project(seg2.src).is_close(seg2.src, r)
                    || seg1.project(seg2.dst).is_close(seg2.dst, r)
                    || seg2.project(seg1.src).is_close(seg1.src, r)
                    || seg2.project(seg1.dst).is_close(seg1.dst, r)
                {
                    return true;
                }
            }
        }
        false
    }
}

defer_inter!(AABB => BoldLine);
impl Intersect<AABB> for BoldLine {
    #[inline]
    fn intersects(&self, p: &AABB) -> bool {
        let big = p.expand(self.radius);
        self.line.segments().any(|x| x.intersects(&big))
    }
}

defer_inter!(OBB => BoldLine);
impl Intersect<OBB> for BoldLine {
    #[inline]
    fn intersects(&self, p: &OBB) -> bool {
        let big = p.expand(self.radius);
        self.line.segments().any(|x| x.intersects(&big))
    }
}
