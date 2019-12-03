use vecmath::Vector2;

struct Human {
    position: Vector2<f32>,
    direction: Vector2<f32>,
}

impl Default for Human {
    fn default() -> Self {
        Human {
            position: [0., 0.].into(),
            direction: [1., 0.].into(),
        }
    }
}

#[derive(Default)]
struct HumanManager {
    humans: Vec<Human>,
}

impl HumanManager {
    fn update(&mut self, delta: f32) {
        //
    }
}
