use crate::context::Context;
use crate::gui::inputmap::{InputAction, InputMap};
use crate::gui::windows::settings::Settings;
use common::saveload::Encoder;
use geom::{vec4, Camera, Matrix4, Plane, Ray3, Vec2, Vec3, AABB};
use wgpu_engine::Tesselator;

pub struct CameraHandler3D {
    pub camera: Camera,
    pub lastscreenpos: Vec2,
    pub last_pos: Option<Vec2>,
    pub targetpos: Vec3,
    pub targetyaw: f32,
    pub targetpitch: f32,
    pub targetdist: f32,
}

impl CameraHandler3D {
    pub fn update(&mut self, ctx: &mut Context) {
        let proj = self.camera.build_view_projection_matrix();
        let inv_proj = proj.invert().unwrap_or_else(Matrix4::zero);

        ctx.gfx.set_proj(proj);
        ctx.gfx.set_inv_proj(inv_proj);
    }

    pub fn height(&self) -> f32 {
        self.camera.offset().z
    }

    pub fn cull_tess(&self, tess: &mut Tesselator) {
        let p = self.camera.pos;
        tess.cull_rect = Some(AABB::new(p.xy(), p.xy()).expand(2000.0));
        tess.zoom = 1000.0 / self.height();
    }

    pub fn follow(&mut self, p: Vec3) {
        self.camera.pos = p;
        self.targetpos = p;
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    pub fn unproject(&self, pos: Vec2, height: impl Fn(Vec2) -> Option<f32>) -> Option<Vec3> {
        let proj = self.camera.build_view_projection_matrix();
        let inv = proj.invert()?;

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
            n: Vec3::Z,
        };

        let p = r.intersection_plane(&p)?.xy();
        Some(p.z(height(p)?))
    }

    fn save(&self) {
        let cam = self.camera;
        rayon::spawn(move || {
            common::saveload::JSON::save_silent(&cam, "camera3D");
        });
    }

    pub fn load(viewport: (u32, u32)) -> Self {
        let camera = common::saveload::JSON::load("camera3D")
            .unwrap_or_else(|| Camera::new(Vec3::ZERO, viewport.0 as f32, viewport.1 as f32));

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
        inps: &InputMap,
        settings: &Settings,
        height: impl Fn(Vec2) -> Option<f32>,
    ) {
        self.save();
        let delta = delta.min(0.1);
        let off = self.camera.offset();
        let d = off.xy().try_normalize().unwrap_or(Vec2::ZERO) * self.camera.dist;
        let screenpos = ctx.input.mouse.screen;

        if inps.act.contains(&InputAction::GoRight) {
            self.targetpos += -delta * d.perpendicular().z0();
        }
        if inps.act.contains(&InputAction::GoLeft) {
            self.targetpos += delta * d.perpendicular().z0();
        }
        if inps.act.contains(&InputAction::GoForward) {
            self.targetpos += -delta * d.z0();
        }
        if inps.act.contains(&InputAction::GoBackward) {
            self.targetpos += delta * d.z0();
        }

        if inps.act.contains(&InputAction::Zoom) {
            self.targetdist *= 1.0 / 1.1;
        }

        if inps.act.contains(&InputAction::Dezoom) {
            self.targetdist *= 1.1;
        }

        if settings.camera_border_move {
            if screenpos.x < 2.0 {
                self.targetpos += delta * d.perpendicular().z0();
            }
            if screenpos.x > self.camera.viewport_w - 2.0 {
                self.targetpos += -delta * d.perpendicular().z0();
            }
            if screenpos.y < 2.0 {
                self.targetpos += -delta * d.z0();
            }
            if screenpos.y > self.camera.viewport_h - 2.0 {
                self.targetpos += delta * d.z0();
            }
        }

        let delta_mouse = screenpos - self.lastscreenpos;
        self.lastscreenpos = screenpos;

        let unprojected = self.unproject(screenpos, |_| Some(0.0));

        if inps.act.contains(&InputAction::CameraRotate) {
            self.targetyaw -= delta_mouse.x / 100.0;
            self.targetpitch += delta_mouse.y / 100.0;
            self.targetpitch = self.targetpitch.min(1.57).max(0.01);
        } else if inps.act.contains(&InputAction::CameraMove) {
            if let Some((last_pos, unprojected)) = self.last_pos.zip(unprojected) {
                self.targetpos += (last_pos - unprojected.xy())
                    .cap_magnitude(50.0 * delta * self.camera.eye().z)
                    .z0();
            }
        }

        self.targetdist = self.targetdist.clamp(5.0, 100000.0);

        if settings.camera_smooth {
            macro_rules! lerpp {
                ($a:expr, $b:expr, $amt:expr) => {
                    let coeff = delta * settings.camera_smooth_tightness * $amt;
                    let diff = $b - $a;
                    if coeff.abs() > 1.0 {
                        $a = $b;
                    } else {
                        $a += diff * coeff;
                    }
                };
            }

            lerpp!(self.camera.pos, self.targetpos, 8.0);
            lerpp!(self.camera.yaw, self.targetyaw, 16.0);
            lerpp!(self.camera.pitch, self.targetpitch, 8.0);
            lerpp!(self.camera.dist, self.targetdist, 8.0);
        } else {
            self.camera.pos = self.targetpos;
            self.camera.yaw = self.targetyaw;
            self.camera.pitch = self.targetpitch;
            self.camera.dist = self.targetdist;
        }

        self.camera.fovy = settings.camera_fov.clamp(1.0, 179.0);

        self.camera.pos.z = height(self.camera.pos.xy()).unwrap_or(self.camera.pos.z);
        self.update(ctx);
        self.last_pos = self.unproject(screenpos, |_| Some(0.0)).map(Vec3::xy);
    }
}
