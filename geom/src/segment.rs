use super::Vec2;
use crate::polygon::Polygon;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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
