use crate::context::Context;
use crate::gui::windows::settings::Settings;
use crate::input::{KeyCode, MouseButton};
use common::saveload::Encoder;
use geom::{vec2, vec3, Camera, Vec2, Vec3};
use wgpu_engine::Tesselator;

pub struct CameraHandler {
    pub camera: Camera,
    pub last_pos: Vec2,
    pub movespeed: f32,
    pub targetpos: Vec3,
}

impl CameraHandler {
    pub fn update(&mut self, ctx: &mut Context) {
        self.camera.update();
        ctx.gfx.set_proj(self.camera.projection());
        ctx.gfx.set_inv_proj(self.camera.inv_projection());
    }

    pub fn cull_tess(&self, tess: &mut Tesselator) {
        tess.cull_rect = Some(self.camera.screen_aabb());
        tess.zoom = 1000.0 / self.camera.position.z;
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    pub fn unproject(&self, pos: Vec2) -> Vec2 {
        self.camera.unproject(pos)
    }

    fn save(&self) {
        common::saveload::JSON::save_silent(&self.targetpos, "camera");
    }
    pub fn load(viewport: (u32, u32)) -> Self {
        let pos = common::saveload::JSON::load("camera").unwrap_or_else(|| vec3(0.0, 0.0, 1000.0));
        Self {
            camera: Camera::new(viewport.0 as f32, viewport.1 as f32, pos),
            last_pos: Default::default(),
            movespeed: 0.8,
            targetpos: pos,
        }
    }

    pub fn camera_movement(
        &mut self,
        ctx: &mut Context,
        delta: f32,
        mouse_enabled: bool,
        keyboard_enabled: bool,
        settings: &Settings,
    ) {
        let delta = delta.min(0.1);
        let p = ctx.input.mouse.unprojected;
        let screenpos = ctx.input.mouse.screen;

        if mouse_enabled {
            if ctx.input.mouse.pressed.contains(&MouseButton::Right)
                || ctx.input.mouse.pressed.contains(&MouseButton::Middle)
            {
                self.targetpos.x -= p.x - self.last_pos.x;
                self.targetpos.y -= p.y - self.last_pos.y;
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
                self.translate_movespeed(delta, vec2(-1.0, 0.0));
            }
            if screenpos.x > self.camera.viewport.x - 2.0 {
                self.translate_movespeed(delta, vec2(1.0, 0.0));
            }
            if screenpos.y < 2.0 {
                self.translate_movespeed(delta, vec2(0.0, 1.0));
            }
            if screenpos.y > self.camera.viewport.y - 2.0 {
                self.translate_movespeed(delta, vec2(0.0, -1.0));
            }
        }

        if keyboard_enabled {
            let is_pressed = &ctx.input.keyboard.pressed;

            if is_pressed.contains(&KeyCode::Right) {
                self.translate_movespeed(delta, vec2(1.0, 0.0));
            }
            if is_pressed.contains(&KeyCode::Left) {
                self.translate_movespeed(delta, vec2(-1.0, 0.0));
            }
            if is_pressed.contains(&KeyCode::Up) {
                self.translate_movespeed(delta, vec2(0.0, 1.0));
            }
            if is_pressed.contains(&KeyCode::Down) {
                self.translate_movespeed(delta, vec2(0.0, -1.0));
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

        if settings.camera_smooth {
            self.camera.position += (self.targetpos - self.camera.position) * delta * 8.0;
        } else {
            self.camera.position = self.targetpos;
        }
        self.update(ctx);

        self.last_pos = self.unproject(ctx.input.mouse.screen);
    }

    fn translate_movespeed(&mut self, delta: f32, dir: Vec2) {
        let m = delta * self.movespeed * self.camera.position.z * dir;
        self.targetpos += m.z(0.0);
        self.save();
    }

    fn zoom_by(&mut self, ctx: &mut Context, multiply: f32, lock: bool) {
        let mut cpy = self.camera;
        cpy.position = self.targetpos;
        cpy.update();
        let before = cpy.unproject(ctx.input.mouse.screen);

        cpy.position.z *= multiply;
        if lock {
            cpy.position.z = cpy.position.z.min(20000.0).max(5.0);
        }

        cpy.update();
        let after = cpy.unproject(ctx.input.mouse.screen);
        cpy.position += (before - after).z(0.0);

        self.targetpos = cpy.position;
        self.save();
    }
}
