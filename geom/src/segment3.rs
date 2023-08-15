use crate::{Line3, Segment, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment3 {
    pub src: Vec3,
    pub dst: Vec3,
}

impl Segment3 {
    pub fn new(src: Vec3, dst: Vec3) -> Self {
        Self { src, dst }
    }

    pub fn project(&self, p: Vec3) -> Vec3 {
        let diff = self.dst - self.src;
        let diff2 = p - self.src;
        let diff3 = p - self.dst;

        let proj1 = diff2.dot(diff);
        let proj2 = -diff3.dot(diff);

        if proj1 <= 0.0 {
            self.src
        } else if proj2 <= 0.0 {
            self.dst
        } else {
            let lol = proj1 / diff.mag2();
            self.src + diff * lol
        }
    }

    pub fn flatten(&self) -> Segment {
        Segment {
            src: self.src.xy(),
            dst: self.dst.xy(),
        }
    }

    pub fn as_line(&self) -> Line3 {
        Line3 {
            src: self.src,
            dst: self.dst,
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

    pub fn scale(&mut self, scale: f32) -> &mut Self {
        self.resize(self.vec().mag() * scale)
    }

    pub fn vec(&self) -> Vec3 {
        self.dst - self.src
    }

    pub fn middle(&self) -> Vec3 {
        (self.src + self.dst) * 0.5
    }
}
