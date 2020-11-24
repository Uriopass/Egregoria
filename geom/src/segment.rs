use super::Vec2;
use crate::polygon::Polygon;
use crate::{Circle, Intersect, Shape, AABB};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Segment {
    pub src: Vec2,
    pub dst: Vec2,
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
            let lol = proj1 / diff.magnitude2();
            self.src + diff * lol
        }
    }

    pub fn resize(&mut self, length: f32) -> &mut Self {
        if let Some(v) = self.vec().try_normalize_to(length) {
            let mid = (self.src + self.dst) * 0.5;
            self.src = mid - v * 0.5;
            self.dst = mid + v * 0.5;
        }
        self
    }

    pub fn vec(&self) -> Vec2 {
        self.dst - self.src
    }

    pub fn to_polygon(self) -> Polygon {
        Polygon(vec![self.src, self.dst])
    }

    pub fn center(&self) -> Vec2 {
        (self.src + self.dst) * 0.5
    }
}

impl Shape for Segment {
    fn bbox(&self) -> AABB {
        AABB::new(self.src, self.dst)
    }
}

impl Intersect<AABB> for Segment {
    fn intersects(&self, aabb: AABB) -> bool {
        aabb.contains(self.src)
            || aabb.contains(self.dst)
            || aabb.segments().any(|s| s.intersects(*self))
    }
}

fn ccw(a: Vec2, b: Vec2, c: Vec2) -> bool {
    (c.y - a.y) * (b.x - a.x) > (b.y - a.y) * (c.x - a.x)
}

impl Intersect<Segment> for Segment {
    fn intersects(&self, s: Segment) -> bool {
        ccw(self.src, s.src, s.dst) != ccw(self.dst, s.src, s.dst)
            && ccw(self.src, self.dst, s.src) != ccw(self.src, self.dst, s.dst)
    }
}

impl Intersect<Circle> for Segment {
    fn intersects(&self, c: Circle) -> bool {
        c.intersects(*self)
    }
}

impl Intersect<Vec2> for Segment {
    fn intersects(&self, _p: Vec2) -> bool {
        false
    }
}
