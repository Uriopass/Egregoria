#![allow(dead_code)]
use crate::context::Context;
use crate::gui::windows::settings::Settings;
use crate::input::KeyCode;
use geom::{mulmatvec, vec2, Camera3D, Vec2, Vec3, AABB};
use wgpu_engine::Tesselator;

pub struct CameraHandler3D {
    pub camera: Camera3D,
}

impl CameraHandler3D {
    pub fn update(&mut self, ctx: &mut Context) {
        let (proj, inv_proj) = self.camera.build_view_projection_matrix();

        ctx.gfx.set_proj(proj);
        ctx.gfx.set_inv_proj(inv_proj);
    }

    pub fn height(&self) -> f32 {
        self.camera.offset.z
    }

    pub fn cull_tess(&self, tess: &mut Tesselator) {
        let p = self.camera.pos;
        tess.cull_rect = Some(AABB::new(
            p - vec2(1000.0, 1000.0),
            p + vec2(1000.0, 1000.0),
        ));
        tess.zoom = 1000.0 / self.height();
    }

    pub fn follow(&mut self, p: Vec3) {
        self.camera.pos = p.xy();
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    pub fn unproject(&self, pos: Vec2) -> Vec2 {
        let (_, inv) = self.camera.build_view_projection_matrix();

        let v = mulmatvec(inv, mint::Vector4::from([pos.x, pos.y, 1.0, 1.0]));
        Vec2 { x: v.x, y: v.y }
    }

    fn save(&self) {}

    pub fn load(viewport: (u32, u32)) -> Self {
        Self {
            camera: Camera3D::new(Vec2::ZERO, viewport.0 as f32, viewport.1 as f32),
        }
    }

    pub fn camera_movement(
        &mut self,
        ctx: &mut Context,
        delta: f32,
        _mouse_enabled: bool,
        keyboard_enabled: bool,
        settings: &Settings,
    ) {
        let delta = delta.min(0.1);
        if keyboard_enabled {
            let is_pressed = &ctx.input.keyboard.pressed;

            let d = self.camera.offset.xy();
            if is_pressed.contains(&KeyCode::Right) {
                self.camera.pos += -delta * d.perpendicular();
            }
            if is_pressed.contains(&KeyCode::Left) {
                self.camera.pos += delta * d.perpendicular();
            }
            if is_pressed.contains(&KeyCode::Up) {
                self.camera.pos += -delta * d;
            }
            if is_pressed.contains(&KeyCode::Down) {
                self.camera.pos += delta * d;
            }

            let just_pressed = &ctx.input.keyboard.just_pressed;
            if just_pressed.contains(&KeyCode::Add) || just_pressed.contains(&KeyCode::Equals) {
                self.camera.offset = self.camera.offset * 1.0 / 1.1;
            }

            if just_pressed.contains(&KeyCode::Subtract) || just_pressed.contains(&KeyCode::Minus) {
                self.camera.offset = self.camera.offset * 1.1;
            }

            if settings.camera_lock {
                self.camera.offset.z = self.camera.offset.z.min(20000.0).max(5.0);
            }
        }

        self.update(ctx);
    }
}
