use crate::engine::MouseButton;
use crate::engine::{Context, KeyCode};
use crate::geometry::Camera;
use crate::geometry::Rect;
use cgmath::{vec2, Vector2};

#[allow(dead_code)]
pub struct CameraHandler {
    pub camera: Camera,
    last_pos: Vector2<f32>,
}

const CAMERA_KEY_MOVESPEED: f32 = 300.0;

#[allow(dead_code)]
impl CameraHandler {
    pub fn new(width: f32, height: f32, zoom: f32) -> CameraHandler {
        CameraHandler {
            camera: Camera::new(width, height, zoom),
            last_pos: vec2(0.0, 0.0),
        }
    }

    pub fn center_camera(&mut self, ctx: &mut Context) {
        self.camera.position.x = 0.0;
        self.camera.position.y = 0.0;
        self.update(ctx);
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.camera.update();
        ctx.gfx.set_proj(self.camera.projection);
    }

    pub fn get_screen_box(&self) -> Rect {
        let upleft = self.camera.unproject([0.0, 0.0].into());
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

    pub fn projection(&self) -> cgmath::Matrix4<f32> {
        self.camera.projection
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    pub fn unproject_mouse_click(&self, pos: Vector2<f32>) -> Vector2<f32> {
        self.camera.unproject(pos)
    }

    pub fn easy_camera_movement(
        &mut self,
        ctx: &mut Context,
        delta: f32,
        mouse_enabled: bool,
        keyboard_enabled: bool,
    ) {
        let p = ctx.input.mouse.unprojected;
        if mouse_enabled && ctx.input.mouse.buttons.contains(&MouseButton::Right) {
            self.camera.position.x -= p.x - self.last_pos.x;
            self.camera.position.y -= p.y - self.last_pos.y;
            self.update(ctx);
        }

        if mouse_enabled {
            self.last_pos = self.unproject_mouse_click(ctx.input.mouse.screen);
            if ctx.input.mouse.wheel_delta > 0.0 {
                self.zoom_by(ctx, 1.1);
            }
            if ctx.input.mouse.wheel_delta < 0.0 {
                self.zoom_by(ctx, 1.0 / 1.1);
            }
        }

        if keyboard_enabled {
            let is_pressed = &ctx.input.keyboard.is_pressed;

            if is_pressed.contains(&KeyCode::Right) {
                self.camera.position.x += delta * CAMERA_KEY_MOVESPEED / self.camera.zoom;
            }
            if is_pressed.contains(&KeyCode::Left) {
                self.camera.position.x -= delta * CAMERA_KEY_MOVESPEED / self.camera.zoom;
            }
            if is_pressed.contains(&KeyCode::Up) {
                self.camera.position.y += delta * CAMERA_KEY_MOVESPEED / self.camera.zoom;
            }
            if is_pressed.contains(&KeyCode::Down) {
                self.camera.position.y -= delta * CAMERA_KEY_MOVESPEED / self.camera.zoom;
            }

            self.last_pos = self.unproject_mouse_click(ctx.input.mouse.screen);
            let just_pressed = &ctx.input.keyboard.just_pressed;
            if just_pressed.contains(&KeyCode::Add) || just_pressed.contains(&KeyCode::Equals) {
                self.zoom_by(ctx, 1.1);
            }

            let just_pressed = &ctx.input.keyboard.just_pressed; // cannot call zoom_by 2 lines above without reborrowing
            if just_pressed.contains(&KeyCode::Subtract) || just_pressed.contains(&KeyCode::Minus) {
                self.zoom_by(ctx, 1.0 / 1.1);
            }
        }
    }

    fn zoom_by(&mut self, ctx: &mut Context, multiply: f32) {
        self.camera.zoom *= multiply;

        self.update(ctx);
        let after = self.unproject_mouse_click(ctx.input.mouse.screen);
        self.camera.position.x -= after.x - self.last_pos.x;
        self.camera.position.y -= after.y - self.last_pos.y;
        self.update(ctx);
    }
}
