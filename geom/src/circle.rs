use crate::{Intersect, Polygon, Segment, Shape, Vec2, AABB, OBB};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    #[inline]
    pub fn contains(&self, p: Vec2) -> bool {
        self.center.is_close(p, self.radius)
    }

    #[inline]
    pub fn moment_of_inertia(&self, mass: f32) -> f32 {
        mass * self.radius * self.radius * 0.5
    }
}

impl Circle {
    #[inline]
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Shape for Circle {
    #[inline]
    fn bbox(&self) -> AABB {
        AABB {
            ll: self.center - Vec2::splat(self.radius),
            ur: self.center + Vec2::splat(self.radius),
        }
    }
}

impl Intersect<AABB> for Circle {
    #[inline]
    fn intersects(&self, b: &AABB) -> bool {
        let proj = self.center.min(b.ur).max(b.ll);
        self.center.is_close(proj, self.radius)
    }
}

impl Intersect<Circle> for Circle {
    #[inline]
    fn intersects(&self, c: &Circle) -> bool {
        self.center.is_close(c.center, self.radius + c.radius)
    }
}

impl Intersect<Segment> for Circle {
    #[inline]
    fn intersects(&self, s: &Segment) -> bool {
        s.project(self.center).is_close(self.center, self.radius)
    }
}

impl Intersect<Vec2> for Circle {
    #[inline]
    fn intersects(&self, p: &Vec2) -> bool {
        self.center.is_close(*p, self.radius)
    }
}

defer_inter!(Circle => OBB);
defer_inter!(Circle => Polygon);
