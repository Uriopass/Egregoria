use super::Vec2;
use crate::Line;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub from: Vec2,
    pub dir: Vec2,
}

impl Ray {
    pub fn intersection_point(&self, r2: &Ray) -> Option<Vec2> {
        let div = self.dir.perp_dot(r2.dir);

        let p_diff = self.from - r2.from;
        let t = r2.dir.perp_dot(p_diff);
        let s = self.dir.perp_dot(p_diff);

        if t * div > 0.0 && s * div > 0.0 {
            Some(self.from + self.dir * t / div)
        } else {
            None
        }
    }

    pub fn as_line(&self) -> Line {
        Line {
            src: self.from,
            dst: self.from + self.dir,
        }
    }

    pub fn both_dist_to_inter(&self, r2: &Ray) -> Option<(f32, f32)> {
        let div = self.dir.perp_dot(r2.dir);

        let p_diff = self.from - r2.from;
        let t = r2.dir.perp_dot(p_diff);
        let s = self.dir.perp_dot(p_diff);

        if t * div > 0.0 && s * div > 0.0 {
            Some((t / div, s / div))
        } else {
            None
        }
    }
}

impl Ray {
    pub fn new(from: Vec2, dir: Vec2) -> Self {
        Ray { from, dir }
    }
}
