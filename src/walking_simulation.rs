use ggez::nalgebra::Vector2;

struct Human {
    position: Vector2<f32>,
}

impl Default for Human {
    fn default() -> Self {
        Human {
            position: Vector2::new(0., 0.),
        }
    }
}
