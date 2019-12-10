use ggez::input;
use ggez::input::keyboard::KeyCode;
use ggez::input::mouse::MouseButton;

use ggez::graphics;
use ggez::Context;

use crate::camera::Camera;
use crate::geometry::rect::Rect;
use cgmath::Vector2;

#[allow(dead_code)]
pub struct CameraHandler {
    pub camera: Camera,
    last_pos: Vector2<f32>,
}

#[allow(dead_code)]
impl CameraHandler {
    pub fn new() -> CameraHandler {
        CameraHandler {
            camera: Camera::new(800., 600.),
            last_pos: [0., 0.].into(),
        }
    }

    pub fn center_camera(&mut self, ctx: &mut Context) {
        self.camera.position.x = 0.;
        self.camera.position.y = 0.;
        self.update(ctx);
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.camera.update();
        graphics::set_projection(ctx, self.camera.projection);
        graphics::apply_transformations(ctx).unwrap();
    }

    pub fn get_screen_box(&self) -> Rect {
        let upleft = self.camera.unproject([0., 0.].into());
        let downright = self
            .camera
            .unproject([self.camera.viewport.x, self.camera.viewport.y].into());
        Rect {
            x: upleft.x,
            y: downright.y,
            w: downright.x - upleft.x,
            h: upleft.y - downright.y,
        }
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    pub fn unproject_mouse_click(&self, ctx: &Context) -> Vector2<f32> {
        let haha = ggez::input::mouse::position(ctx);
        self.camera.unproject(Vector2::new(haha.x, haha.y))
    }

    pub fn easy_camera_movement(&mut self, ctx: &mut Context) {
        let p = self.unproject_mouse_click(ctx);
        if input::mouse::button_pressed(ctx, MouseButton::Right) {
            self.camera.position.x -= p.x - self.last_pos.x;
            self.camera.position.y -= p.y - self.last_pos.y;
            self.update(ctx);
        }
        self.last_pos = self.unproject_mouse_click(ctx);
    }

    pub fn easy_camera_movement_keys(&mut self, ctx: &mut Context, keycode: KeyCode) {
        if keycode == KeyCode::Add || keycode == KeyCode::Subtract {
            let before = self.unproject_mouse_click(ctx);
            if keycode == KeyCode::Add {
                self.camera.zoom *= 1.2;
            } else {
                self.camera.zoom /= 1.2;
            }
            self.update(ctx);
            let after = self.unproject_mouse_click(ctx);
            self.camera.position.x -= after.x - before.x;
            self.camera.position.y -= after.y - before.y;
            self.update(ctx);
        }
    }
}
