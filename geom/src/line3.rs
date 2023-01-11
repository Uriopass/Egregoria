use crate::Vec3;

pub struct Line3 {
    pub src: Vec3,
    pub dst: Vec3,
}

impl Line3 {
    pub fn new(src: Vec3, dst: Vec3) -> Self {
        Self { src, dst }
    }

    pub fn project(&self, p: Vec3) -> Vec3 {
        let r = self.vec();
        let diff2 = p - self.src;

        let proj1 = diff2.dot(r);

        let d = proj1 / r.mag2();
        self.src + r * d
    }

    pub fn vec(&self) -> Vec3 {
        self.dst - self.src
    }
}
