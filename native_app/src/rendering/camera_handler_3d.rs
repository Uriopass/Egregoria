use crate::context::Context;
use crate::gui::windows::settings::Settings;
use crate::input::{KeyCode, MouseButton};
use common::saveload::Encoder;
use geom::{vec4, Camera, Matrix4, Plane, Ray3, Vec2, Vec3, AABB};
use wgpu_engine::Tesselator;

pub struct CameraHandler3D {
    pub camera: Camera,
    pub lastscreenpos: Vec2,
    pub last_pos: Vec2,
    pub targetpos: Vec2,
    pub targetyaw: f32,
    pub targetpitch: f32,
    pub targetdist: f32,
}

impl CameraHandler3D {
    pub fn update(&mut self, ctx: &mut Context) {
        let proj = self.camera.build_view_projection_matrix();
        let inv_proj = proj.invert().unwrap_or_else(|| Matrix4::from([0.0; 16]));

        ctx.gfx.set_proj(proj);
        ctx.gfx.set_inv_proj(inv_proj);
    }

    pub fn height(&self) -> f32 {
        self.camera.offset().z
    }

    pub fn cull_tess(&self, tess: &mut Tesselator) {
        let p = self.camera.pos;
        tess.cull_rect = Some(AABB::new(p, p).expand(2000.0));
        tess.zoom = 1000.0 / self.height();
    }

    pub fn follow(&mut self, p: Vec3) {
        self.camera.pos = p.xy();
        self.targetpos = p.xy();
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    pub fn unproject(&self, pos: Vec2) -> Vec2 {
        let proj = self.camera.build_view_projection_matrix();
        let inv = proj.invert().unwrap();

        let v = inv
            * vec4(
                2.0 * pos.x / self.camera.viewport_w - 1.0,
                -(2.0 * pos.y / self.camera.viewport_h - 1.0),
                1.0,
                1.0,
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
        let camera = common::saveload::JSON::load("camera3D")
            .unwrap_or_else(|| Camera::new(Vec2::ZERO, viewport.0 as f32, viewport.1 as f32));

        Self {
            camera,
            lastscreenpos: Default::default(),
            last_pos: Default::default(),
            targetpos: camera.pos,
            targetyaw: camera.yaw,
            targetpitch: camera.pitch,
            targetdist: camera.dist,
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
        let off = self.camera.offset();
        let d = off.xy().try_normalize().unwrap_or(Vec2::ZERO) * self.camera.dist;
        let screenpos = ctx.input.mouse.screen;

        if keyboard_enabled {
            let pressed = &ctx.input.keyboard.pressed;

            if pressed.contains(&KeyCode::Right) {
                self.targetpos += -delta * d.perpendicular();
            }
            if pressed.contains(&KeyCode::Left) {
                self.targetpos += delta * d.perpendicular();
            }
            if pressed.contains(&KeyCode::Up) {
                self.targetpos += -delta * d;
            }
            if pressed.contains(&KeyCode::Down) {
                self.targetpos += delta * d;
            }

            let just_pressed = &ctx.input.keyboard.just_pressed;
            if just_pressed.contains(&KeyCode::Add) || just_pressed.contains(&KeyCode::Equals) {
                self.targetdist *= 1.0 / 1.1;
            }

            if just_pressed.contains(&KeyCode::Subtract) || just_pressed.contains(&KeyCode::Minus) {
                self.targetdist *= 1.1;
            }
        }

        if settings.camera_border_move {
            if screenpos.x < 2.0 {
                self.targetpos += delta * d.perpendicular();
            }
            if screenpos.x > self.camera.viewport_w - 2.0 {
                self.targetpos += -delta * d.perpendicular();
            }
            if screenpos.y < 2.0 {
                self.targetpos += -delta * d;
            }
            if screenpos.y > self.camera.viewport_h - 2.0 {
                self.targetpos += delta * d;
            }
        }

        let delta_mouse = screenpos - self.lastscreenpos;
        self.lastscreenpos = screenpos;

        let unprojected = ctx.input.mouse.unprojected;

        if mouse_enabled {
            if ctx.input.mouse.wheel_delta < 0.0 {
                self.targetdist *= 1.1;
            }
            if ctx.input.mouse.wheel_delta > 0.0 {
                self.targetdist *= 1.0 / 1.1;
            }
            let pressed = &ctx.input.mouse.pressed;

            if (pressed.contains(&MouseButton::Right)
                && ctx.input.keyboard.pressed.contains(&KeyCode::LShift))
                || pressed.contains(&MouseButton::Middle)
            {
                self.targetyaw -= delta_mouse.x / 100.0;
                self.targetpitch += delta_mouse.y / 100.0;
                self.targetpitch = self.targetpitch.min(1.57).max(0.01);
            } else if pressed.contains(&MouseButton::Right) {
                self.targetpos += (self.last_pos - unprojected).cap_magnitude(50000.0 * delta);
                self.targetpos.x = self.targetpos.x.clamp(-10000.0, 10000.0);
                self.targetpos.y = self.targetpos.y.clamp(-10000.0, 10000.0);
            }
        }
        self.targetdist = self.targetdist.clamp(30.0, 100000.0);

        if settings.camera_smooth {
            self.camera.pos +=
                (self.targetpos - self.camera.pos) * delta * settings.camera_smooth_tightness * 8.0;
            self.camera.yaw += (self.targetyaw - self.camera.yaw)
                * delta
                * settings.camera_smooth_tightness
                * 16.0;
            self.camera.pitch += (self.targetpitch - self.camera.pitch)
                * delta
                * settings.camera_smooth_tightness
                * 8.0;
            self.camera.dist += (self.targetdist - self.camera.dist)
                * delta
                * settings.camera_smooth_tightness
                * 8.0;
        } else {
            self.camera.pos = self.targetpos;
            self.camera.yaw = self.targetyaw;
            self.camera.pitch = self.targetpitch;
            self.camera.dist = self.targetdist;
        }

        self.update(ctx);
        self.last_pos = self.unproject(screenpos);
    }
}
