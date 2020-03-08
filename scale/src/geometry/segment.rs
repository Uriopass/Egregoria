use cgmath::InnerSpace;
use cgmath::Vector2;

pub struct Segment {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
}

impl Segment {
    pub fn new(a: Vector2<f32>, b: Vector2<f32>) -> Self {
        Self { a, b }
    }

    pub fn project(&self, p: Vector2<f32>) -> Vector2<f32> {
        let diff: Vector2<f32> = self.b - self.a;
        let diff2: Vector2<f32> = p - self.a;
        let diff3: Vector2<f32> = p - self.b;

        let proj1 = diff2.dot(diff);
        let proj2 = diff3.dot(-diff);

        if proj1 <= 0.0 {
            self.a
        } else if proj2 <= 0.0 {
            self.b
        } else {
            self.a + diff * (proj1 / diff.magnitude())
        }
    }
}
