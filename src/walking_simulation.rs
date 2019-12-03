use crate::shape_render::ShapeRenderer;
use cgmath::{EuclideanSpace, Point2, Vector2, Zero};

struct Human {
    position: Point2<f32>,
    direction: Vector2<f32>,
}

impl Default for Human {
    fn default() -> Self {
        Human {
            position: Point2::<f32>::origin(),
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

    fn draw(&self, sr: &mut ShapeRenderer) {
        for human in self.humans.iter() {}
    }
}
