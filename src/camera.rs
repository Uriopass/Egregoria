use ggez::nalgebra::*;

pub struct Camera {
    viewport: Vector2<f32>,
    position: Vector2<f32>,

    pub zoom: f32,
    pub projection: Matrix4<f32>,
}

impl Camera {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Camera {
        let mut c = Camera {
            viewport: Vector2::new(viewport_width, viewport_height),

            position: Vector2::new(0.0, 0.0),
            projection: Matrix4::zeros(),
            zoom: 1.0,
        };
        c.update();
        c
    }

    pub fn update(&mut self) {
        self.projection = Matrix4::new(
            1. / self.viewport.x,
            0.,
            0.,
            -1. - self.position.x / self.viewport.x,
            0.,
            1. / self.viewport.y,
            0.,
            -1. - self.position.y / self.viewport.y,
            0.,
            0.,
            1.,
            0.,
            0.,
            0.,
            0.,
            1.,
        );
    }
    #[allow(dead_code)]
    pub fn translate(&mut self, x: f32, y: f32) {
        self.position += Vector2::new(x, y);
    }
    #[allow(dead_code)]
    pub fn unproject(&self, _screen_coords: &Vector2<f32>) -> Vector2<f32> {
        Vector2::zeros()
    }
    #[allow(dead_code)]
    pub fn project(&self, _world_coords: &Vector2<f32>) -> Vector2<f32> {
        Vector2::zeros()
    }

    pub fn set_viewport(&mut self, viewport_width: f32, viewport_height: f32) {
        self.viewport.x = viewport_width;
        self.viewport.y = viewport_height;
    }
}
