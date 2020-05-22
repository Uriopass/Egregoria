use cgmath::{Matrix4, SquareMatrix, Vector2, Vector4, Zero};

pub struct Camera {
    pub viewport: Vector2<f32>,
    pub position: Vector2<f32>,
    pub zoom: f32,
    pub projection: Matrix4<f32>,
    pub invprojection: Matrix4<f32>,
}

impl Camera {
    pub fn new(viewport_width: f32, viewport_height: f32, zoom: f32) -> Camera {
        let mut c = Camera {
            viewport: Vector2::new(viewport_width, viewport_height),
            position: Vector2::zero(),
            projection: Matrix4::zero(),
            invprojection: Matrix4::zero(),
            zoom,
        };
        c.update();
        c
    }

    #[rustfmt::skip]
    pub fn update(&mut self) {
        // If you have rendering problem on mac os, it doesnt come from the projection
        // but don't forget to do cam.update at least once (dont suppose resize will be called)

        let scalex = 2.0 * self.zoom / self.viewport.x;
        let scaley = 2.0 * self.zoom / self.viewport.y;
        let offsetx = -2.0 * self.zoom * self.position.x / self.viewport.x;
        let offsety = -2.0 * self.zoom * self.position.y / self.viewport.y;


        // cgmath matrix init is backwards
        self.projection = Matrix4::new(scalex, 0.0, 0.0, 0.0,
                                       0.0, scaley, 0.0, 0.0,
                                       0.0, 0.0, 0.1, 0.0,
                                       offsetx, offsety, 0.0, 1.0);
        self.invprojection = self.projection.invert().unwrap();
    }

    pub fn unproject(&self, screen_coords: Vector2<f32>) -> Vector2<f32> {
        let v = self.invprojection
            * Vector4::new(
                -1.0 + 2.0 * screen_coords.x / self.viewport.x,
                1.0 - 2.0 * screen_coords.y / self.viewport.y,
                0.0,
                1.0,
            );
        Vector2::new(v.x, v.y)
    }

    #[allow(dead_code)]
    pub fn project(&self, world_coords: Vector2<f32>) -> Vector2<f32> {
        let v = self.projection * Vector4::new(world_coords.x, world_coords.y, 0.0, 1.0);
        Vector2::new(v.x, v.y)
    }

    pub fn set_viewport(&mut self, viewport_width: f32, viewport_height: f32) {
        self.viewport = Vector2::new(viewport_width, viewport_height);
        self.update()
    }
}
