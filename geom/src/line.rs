use crate::{Vec2, Vec2d};

pub struct Line {
    pub src: Vec2,
    pub dst: Vec2,
}

pub struct Lined {
    pub src: Vec2d,
    pub dst: Vec2d,
}

impl Line {
    pub fn new(src: Vec2, dst: Vec2) -> Self {
        Self { src, dst }
    }

    pub fn new_dir(src: Vec2, dir: Vec2) -> Self {
        Self {
            src,
            dst: src + dir,
        }
    }

    pub fn intersection_point(&self, other: &Self) -> Option<Vec2> {
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

        let d = proj1 / r.mag2();
        self.src + r * d
    }

    pub fn vec(&self) -> Vec2 {
        self.dst - self.src
    }
}

impl Lined {
    pub fn new(src: Vec2d, dst: Vec2d) -> Self {
        Self { src, dst }
    }

    pub fn intersection_point(&self, other: &Self) -> Option<Vec2d> {
        // see https://stackoverflow.com/a/565282
        let r = self.vec();
        let s = other.vec();

        let r_cross_s = Vec2d::cross(r, s);
        let q_minus_p = other.src - self.src;

        if r_cross_s != 0.0 {
            let t = Vec2d::cross(q_minus_p, s / r_cross_s);

            return Some(self.src + r * t);
        }
        None
    }

    pub fn project(&self, p: Vec2d) -> Vec2d {
        let r = self.vec();
        let diff2 = p - self.src;

        let proj1 = diff2.dot(r);

        let d = proj1 / r.magnitude2();
        self.src + r * d
    }

    pub fn vec(&self) -> Vec2d {
        self.dst - self.src
    }
}
