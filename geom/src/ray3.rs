use super::Vec3;
use crate::Plane;

#[derive(Debug, Copy, Clone)]
pub struct Ray3 {
    pub from: Vec3,
    pub dir: Vec3,
}

impl Ray3 {
    pub fn new(from: Vec3, dir: Vec3) -> Self {
        Ray3 { from, dir }
    }

    pub fn intersection_plane(&self, p: &Plane) -> Option<Vec3> {
        // assuming vectors are all normalized
        let denom = p.n.dot(self.dir);
        if denom.abs() <= 1e-6 {
            return None;
        }
        let t = (-self.from.dot(p.n) - p.o) / denom;
        if t < 0.0 {
            return None;
        }
        Some(self.from + self.dir * t)
    }
}
