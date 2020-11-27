use crate::{Intersect, Segment, Shape, Vec2, AABB};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Shape for Circle {
    fn bbox(&self) -> AABB {
        AABB {
            ll: self.center - Vec2::splat(self.radius),
            ur: self.center + Vec2::splat(self.radius),
        }
    }
}

impl Intersect<AABB> for Circle {
    fn intersects(&self, b: &AABB) -> bool {
        let d = self.center.min(b.ur).max(b.ll) - self.center;
        d.magnitude2() < self.radius * self.radius
    }
}

impl Intersect<Circle> for Circle {
    fn intersects(&self, c: &Circle) -> bool {
        self.center.is_close(c.center, self.radius + c.radius)
    }
}

impl Intersect<Segment> for Circle {
    fn intersects(&self, s: &Segment) -> bool {
        s.project(self.center).is_close(self.center, self.radius)
    }
}

impl Intersect<Vec2> for Circle {
    fn intersects(&self, p: &Vec2) -> bool {
        self.center.is_close(*p, self.radius)
    }
}
