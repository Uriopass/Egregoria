use crate::{vec2, Vec2, Vec3, AABB};
use mint::ColumnMatrix4;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Camera {
    pub viewport: Vec2,
    pub position: Vec3,
    scale: Vec2,
    offset: Vec2,
}

impl Camera {
    pub fn new(viewport_width: f32, viewport_height: f32, position: Vec3) -> Camera {
        let mut c = Camera {
            viewport: vec2(viewport_width, viewport_height),
            position,
            scale: Vec2::ZERO,
            offset: Vec2::ZERO,
        };
        c.update();
        c
    }

    #[rustfmt::skip]
    pub fn update(&mut self) {
        // If you have rendering problem on mac os, it doesnt come from the projection
        // but don't forget to do cam.update at least once (dont suppose resize will be called)

        self.scale = 2.0 * 1000.0 / (self.position.z * self.viewport);
        self.offset = -2.0 * 1000.0 * vec2(self.position.x, self.position.y) / (self.position.z * self.viewport);
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


    #[rustfmt::skip]
    pub fn inv_projection(&self) -> ColumnMatrix4<f32> {
        ColumnMatrix4::from([1.0 / self.scale.x, 0.0, 0.0, 0.0,
            0.0, 1.0 / self.scale.y, 0.0, 0.0,
            0.0, 0.0, 0.1, 0.0,
            -self.offset.x / self.scale.x, -self.offset.y / self.scale.y, 0.0, 1.0])
    }

    pub fn get_screen_box(&self) -> AABB {
        let ll = self.unproject([0.0, self.viewport.y].into());
        let ur = self.unproject([self.viewport.x, 0.0].into());
        AABB { ll, ur }
    }

    pub fn set_viewport(&mut self, viewport_width: f32, viewport_height: f32) {
        self.viewport = vec2(viewport_width, viewport_height);
        self.update()
    }
}
