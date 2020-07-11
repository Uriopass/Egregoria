use super::Vec2;

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
        let proj2 = diff3.dot(-diff);

        if proj1 <= 0.0 {
            self.src
        } else if proj2 <= 0.0 {
            self.dst
        } else {
            let lol = proj1 / diff.magnitude2();
            self.src + diff * lol
        }
    }
}
