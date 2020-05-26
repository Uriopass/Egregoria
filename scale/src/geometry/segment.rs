use super::Vec2;

pub struct Segment {
    pub a: Vec2,
    pub b: Vec2,
}

impl Segment {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self { a, b }
    }

    pub fn project(&self, p: Vec2) -> Vec2 {
        let diff: Vec2 = self.b - self.a;
        let diff2: Vec2 = p - self.a;
        let diff3: Vec2 = p - self.b;

        let proj1 = diff2.dot(diff);
        let proj2 = diff3.dot(-diff);

        if proj1 <= 0.0 {
            self.a
        } else if proj2 <= 0.0 {
            self.b
        } else {
            let lol = proj1 / diff.magnitude2();
            self.a + diff * lol
        }
    }
}
