#![allow(dead_code)]
use crate::context::Context;
use crate::gui::windows::settings::Settings;
use crate::input::{KeyCode, MouseButton};
use common::saveload::Encoder;
use geom::{mulmatvec, vec2, Camera3D, Plane, Ray3, Vec2, Vec3, AABB};
use wgpu_engine::Tesselator;

pub struct CameraHandler3D {
    pub camera: Camera3D,
    pub lastscreenpos: Vec2,
}

impl CameraHandler3D {
    pub fn update(&mut self, ctx: &mut Context) {
        let (proj, inv_proj) = self.camera.build_view_projection_matrix();

        ctx.gfx.set_proj(proj);
        ctx.gfx.set_inv_proj(inv_proj);
    }

    pub fn height(&self) -> f32 {
        self.camera.offset().z
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

        let v = mulmatvec(
            inv,
            mint::Vector4::from([
                2.0 * pos.x / self.camera.viewport_w - 1.0,
                -(2.0 * pos.y / self.camera.viewport_h - 1.0),
                1.0,
                1.0,
            ]),
        );

        let v = Vec3 {
            x: v.x / v.w,
            y: v.y / v.w,
            z: v.z / v.w,
        } - self.camera.eye();
        let r = Ray3 {
            from: self.camera.eye(),
            dir: v.normalize(),
        };

        let p = Plane {
            p: Vec3::ZERO,
            n: Vec3::UNIT_Z,
        };

        let hit = r.intersection_plane(&p);

        if let Some(hit) = hit {
            return hit.xy();
        }
        Vec2::ZERO
    }

    fn save(&self) {
        let cam = self.camera;
        rayon::spawn(move || {
            common::saveload::JSON::save_silent(&cam, "camera3D");
        });
    }

    pub fn load(viewport: (u32, u32)) -> Self {
        let cam = common::saveload::JSON::load("camera3D");

        Self {
            camera: cam
                .unwrap_or_else(|| Camera3D::new(Vec2::ZERO, viewport.0 as f32, viewport.1 as f32)),
            lastscreenpos: Default::default(),
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
        self.save();
        let delta = delta.min(0.1);
        if keyboard_enabled {
            let pressed = &ctx.input.keyboard.pressed;

            let off = self.camera.offset();
            let d = off.xy().try_normalize().unwrap_or(Vec2::ZERO) * self.camera.dist;
            if pressed.contains(&KeyCode::Right) {
                self.camera.pos += -delta * d.perpendicular();
            }
            if pressed.contains(&KeyCode::Left) {
                self.camera.pos += delta * d.perpendicular();
            }
            if pressed.contains(&KeyCode::Up) {
                self.camera.pos += -delta * d;
            }
            if pressed.contains(&KeyCode::Down) {
                self.camera.pos += delta * d;
            }

            let just_pressed = &ctx.input.keyboard.just_pressed;
            if just_pressed.contains(&KeyCode::Add) || just_pressed.contains(&KeyCode::Equals) {
                self.camera.dist *= 1.0 / 1.1;
            }

            if just_pressed.contains(&KeyCode::Subtract) || just_pressed.contains(&KeyCode::Minus) {
                self.camera.dist *= 1.1;
            }

            if settings.camera_lock {
                self.camera.dist = self.camera.dist.min(20000.0).max(5.0);
            }
        }
        let delta_mouse = ctx.input.mouse.screen - self.lastscreenpos;
        self.lastscreenpos = ctx.input.mouse.screen;

        if mouse_enabled {
            if ctx.input.mouse.wheel_delta < 0.0 {
                self.camera.dist *= 1.1;
            }
            if ctx.input.mouse.wheel_delta > 0.0 {
                self.camera.dist *= 1.0 / 1.1;
            }
            let pressed = &ctx.input.mouse.pressed;

            if pressed.contains(&MouseButton::Middle) {
                self.camera.yaw -= delta_mouse.x / 100.0;
                self.camera.pitch += delta_mouse.y / 100.0;
                self.camera.pitch = self.camera.pitch.min(1.57).max(0.1);
            }
        }

        self.update(ctx);
    }
}
