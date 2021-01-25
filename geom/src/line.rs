use crate::Vec2;

pub struct Line {
    pub src: Vec2,
    pub dst: Vec2,
}

impl Line {
    pub fn new(src: Vec2, dst: Vec2) -> Self {
        Self { src, dst }
    }

    pub fn intersection_point(&self, other: &Line) -> Option<Vec2> {
        // see https://stackoverflow.com/a/565282
        let r = self.vec();
        let s = other.vec();

        let r_cross_s = Vec2::cross(r, s);
        let q_minus_p = other.src - self.src;

        if r_cross_s != 0.0 {
            let t = Vec2::cross(q_minus_p, s / r_cross_s);

            return Some(self.src + r * t);
        }
        None
    }

    pub fn project(&self, p: Vec2) -> Vec2 {
        let r = self.vec();
        let diff2 = p - self.src;

        let proj1 = diff2.dot(r);

        let d = proj1 / r.magnitude2();
        self.src + r * d
    }

    pub fn vec(&self) -> Vec2 {
        self.dst - self.src
    }
}
