use ggez::mint::{Point2, Vector2};
use ggez::nalgebra;
use ggez::nalgebra::Matrix4;

pub struct Camera {
    pub viewport: Vector2<f32>,
    pub position: Point2<f32>,
    pub zoom: f32,
    pub projection: Matrix4<f32>,
    pub invprojection: Matrix4<f32>,
}

impl Camera {
    pub fn new(viewport_width: f32, viewport_height: f32) -> Camera {
        let mut c = Camera {
            viewport: [viewport_width, viewport_height].into(),

            position: [0.0, 0.0].into(),
            projection: Matrix4::zeros(),
            invprojection: Matrix4::zeros(),
            zoom: 1.0,
        };
        c.update();
        c
    }

    #[rustfmt::skip]
    pub fn update(&mut self) {
        let scalex = 2. * self.zoom / self.viewport.x;
        let scaley = 2. * self.zoom / self.viewport.y;
        let offsetx = - 2. * self.zoom * self.position.x / self.viewport.x;
        let offsety = - 2. * self.zoom * self.position.y / self.viewport.y;

        self.projection = Matrix4::new(scalex, 0.,     0., offsetx,
                                       0.,     scaley, 0., offsety,
                                       0.,     0.,     1., 0.,
                                       0.,     0.,     0., 1.);
        self.invprojection = self.projection.try_inverse().unwrap();
    }

    #[allow(dead_code)]
    pub fn translate(&mut self, x: f32, y: f32) {
        self.position.x += x;
        self.position.y += y;
    }

    #[allow(dead_code)]
    pub fn unproject(&self, screen_coords: Point2<f32>) -> cgmath::Vector2<f32> {
        let v = self.invprojection
            * nalgebra::Vector4::new(
                -1. + 2. * screen_coords.x / self.viewport.x,
                1. - 2. * screen_coords.y / self.viewport.y,
                0.0,
                1.0,
            );
        [v.x, v.y].into()
    }

    #[allow(dead_code)]
    pub fn project(&self, world_coords: Point2<f32>) -> Point2<f32> {
        let v = self.projection * nalgebra::Vector4::new(world_coords.x, world_coords.y, 0.0, 1.0);
        [v.x, v.y].into()
    }

    pub fn set_viewport(&mut self, viewport_width: f32, viewport_height: f32) {
        self.viewport = [viewport_width, viewport_height].into();
    }
}
