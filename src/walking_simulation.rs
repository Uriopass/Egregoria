use crate::shape_render::ShapeRenderer;
use cgmath::{Point2, Vector2};
use rand::prelude::SmallRng;
use rand::SeedableRng;

type Vector2f = Vector2<f32>;

#[allow(dead_code)]
struct Human {
    position: Point2<f32>,
    speed: Vector2f,
    direction: Vector2f,
}

impl Default for Human {
    fn default() -> Self {
        Human {
            position: [0., 0.].into(),
            speed: [0., 0.].into(),
            direction: [1., 0.].into(),
        }
    }
}

impl Human {
    fn calc_acceleration(&self) -> Vector2f {
        let a = rand::random::<f32>() - 0.5;
        let b = rand::random::<f32>() - 0.5;
        let r: Vector2f = [a, b].into();
        r * 10.
    }
}

pub struct HumanManager {
    humans: Vec<Human>,
}

impl HumanManager {
    pub fn new(n_humans: i32) -> Self {
        HumanManager {
            humans: (0..n_humans)
                .map(|_| Human {
                    position: [rand::random::<f32>() * 1000., rand::random::<f32>() * 1000.].into(),
                    speed: [0., 0.].into(),
                    direction: [1., 0.].into(),
                })
                .collect(),
        }
    }

    pub fn update(&mut self, delta: f32) {
        for h in self.humans.iter_mut() {
            let acc = h.calc_acceleration();
            h.speed += acc * delta;
            h.position += h.speed * delta;
        }
    }

    pub fn draw(&self, sr: &mut ShapeRenderer) {
        for human in self.humans.iter() {
            sr.draw_circle([human.position.x, human.position.y], 10.);
        }
    }
}
