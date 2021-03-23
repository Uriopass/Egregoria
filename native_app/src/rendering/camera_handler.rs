use crate::context::Context;
use crate::gui::windows::settings::Settings;
use crate::input::{KeyCode, MouseButton};
use common::saveload::Encoder;
use geom::{vec2, Camera, Vec2, Vec3};
use wgpu_engine::Tesselator;

pub struct CameraHandler {
    pub camera: Camera,
    pub last_pos: Vec2,
    pub movespeed: f32,
}

impl CameraHandler {
    pub fn new(width: f32, height: f32, position: Vec3) -> CameraHandler {
        CameraHandler {
            camera: Camera::new(width, height, position),
            last_pos: vec2(0.0, 0.0),
            movespeed: 0.8,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.camera.update();
        ctx.gfx.set_proj(self.camera.projection());
        ctx.gfx.set_inv_proj(self.camera.inv_projection());
    }

    pub fn culled_tesselator(&self) -> Tesselator {
        Tesselator::new(
            Some(self.camera.screen_aabb()),
            1000.0 / self.camera.position.z,
        )
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    pub fn unproject(&self, pos: Vec2) -> Vec2 {
        self.camera.unproject(pos)
    }

    fn save(&self) {
        common::saveload::JSON::save_silent(&self.camera, "camera");
    }

    pub fn camera_movement(
        &mut self,
        ctx: &mut Context,
        delta: f32,
        mouse_enabled: bool,
        keyboard_enabled: bool,
        settings: &Settings,
    ) {
        let p = ctx.input.mouse.unprojected;
        let screenpos = ctx.input.mouse.screen;

        if mouse_enabled {
            if ctx.input.mouse.buttons.contains(&MouseButton::Right)
                || ctx.input.mouse.buttons.contains(&MouseButton::Middle)
            {
                self.camera.position.x -= p.x - self.last_pos.x;
                self.camera.position.y -= p.y - self.last_pos.y;
                self.camera.update();
                self.save();
            }

            self.last_pos = self.unproject(ctx.input.mouse.screen);
            if ctx.input.mouse.wheel_delta < 0.0 {
                self.zoom_by(ctx, 1.1, settings.camera_lock);
            }
            if ctx.input.mouse.wheel_delta > 0.0 {
                self.zoom_by(ctx, 1.0 / 1.1, settings.camera_lock);
            }
        }

        if settings.camera_border_move {
            if screenpos.x < 2.0 {
                self.translate_smooth(delta, vec2(-1.0, 0.0));
            }
            if screenpos.x > self.camera.viewport.x - 2.0 {
                self.translate_smooth(delta, vec2(1.0, 0.0));
            }
            if screenpos.y < 2.0 {
                self.translate_smooth(delta, vec2(0.0, 1.0));
            }
            if screenpos.y > self.camera.viewport.y - 2.0 {
                self.translate_smooth(delta, vec2(0.0, -1.0));
            }
        }

        if keyboard_enabled {
            let is_pressed = &ctx.input.keyboard.is_pressed;

            if is_pressed.contains(&KeyCode::Right) {
                self.translate_smooth(delta, vec2(1.0, 0.0));
            }
            if is_pressed.contains(&KeyCode::Left) {
                self.translate_smooth(delta, vec2(-1.0, 0.0));
            }
            if is_pressed.contains(&KeyCode::Up) {
                self.translate_smooth(delta, vec2(0.0, 1.0));
            }
            if is_pressed.contains(&KeyCode::Down) {
                self.translate_smooth(delta, vec2(0.0, -1.0));
            }

            let just_pressed = &ctx.input.keyboard.just_pressed;
            if just_pressed.contains(&KeyCode::Add) || just_pressed.contains(&KeyCode::Equals) {
                self.zoom_by(ctx, 1.1, settings.camera_lock);
            }

            let just_pressed = &ctx.input.keyboard.just_pressed; // cannot call zoom_by 2 lines above without reborrowing
            if just_pressed.contains(&KeyCode::Subtract) || just_pressed.contains(&KeyCode::Minus) {
                self.zoom_by(ctx, 1.0 / 1.1, settings.camera_lock);
            }
        }

        self.last_pos = self.unproject(ctx.input.mouse.screen);
    }

    fn translate_smooth(&mut self, delta: f32, dir: Vec2) {
        let m = delta * self.movespeed * self.camera.position.z * dir;
        self.camera.position.x += m.x;
        self.camera.position.y += m.y;
        self.camera.update();
        self.save();
    }

    fn zoom_by(&mut self, ctx: &mut Context, multiply: f32, lock: bool) {
        self.camera.position.z *= multiply;
        if lock {
            self.camera.position.z = self.camera.position.z.min(20000.0).max(5.0);
        }

        self.update(ctx);
        let after = self.unproject(ctx.input.mouse.screen);
        self.camera.position.x -= after.x - self.last_pos.x;
        self.camera.position.y -= after.y - self.last_pos.y;
        self.update(ctx);
        self.save();
    }
}
