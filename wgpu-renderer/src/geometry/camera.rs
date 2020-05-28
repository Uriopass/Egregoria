use mint::ColumnMatrix4;
use scale::geometry::{vec2, Vec2};

pub struct Camera {
    pub viewport: Vec2,
    pub position: Vec2,
    pub zoom: f32,
    scale: Vec2,
    offset: Vec2,
}

impl Camera {
    pub fn new(viewport_width: f32, viewport_height: f32, zoom: f32) -> Camera {
        let mut c = Camera {
            viewport: vec2(viewport_width, viewport_height),
            position: Vec2::zero(),
            scale: Vec2::zero(),
            offset: Vec2::zero(),
            zoom,
        };
        c.update();
        c
    }

    #[rustfmt::skip]
    pub fn update(&mut self) {
        // If you have rendering problem on mac os, it doesnt come from the projection
        // but don't forget to do cam.update at least once (dont suppose resize will be called)

        self.scale = 2.0 * self.zoom / self.viewport;
        self.offset = -2.0 * self.zoom * self.position / self.viewport;

/*
        self.projection = Matrix4::new(scalex, 0.0, 0.0, 0.0,
                                       0.0, scaley, 0.0, 0.0,
                                       0.0, 0.0, 0.1, 0.0,
                                       offsetx, offsety, 0.0, 1.0);
        self.invprojection = self.projection.invert().unwrap();
        
 */
    }

    pub fn unproject(&self, screen_coords: Vec2) -> Vec2 {
        let v2 = vec2(
            -1.0 + 2.0 * screen_coords.x / self.viewport.x,
            1.0 - 2.0 * screen_coords.y / self.viewport.y,
        );
        (v2 - self.offset) / self.scale
    }

    #[rustfmt::skip]
    pub fn projection(&self) -> ColumnMatrix4<f32> {
        ColumnMatrix4::from([self.scale.x, 0.0, 0.0, 0.0,
                            0.0, self.scale.y, 0.0, 0.0,
                            0.0, 0.0, 0.1, 0.0,
                            self.offset.x, self.offset.y, 0.0, 1.0])
    }

    #[allow(dead_code)]
    pub fn project(&self, world_coords: Vec2) -> Vec2 {
        world_coords * self.scale + self.offset
    }

    pub fn set_viewport(&mut self, viewport_width: f32, viewport_height: f32) {
        self.viewport = vec2(viewport_width, viewport_height);
        self.update()
    }
}
