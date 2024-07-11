#![allow(clippy::redundant_closure_call)]

use common::saveload::Encoder;
use engine::{Context, Tesselator};
use geom::{Camera, Plane, Radians, Vec2, Vec3, AABB};
use simulation::map::pathfinding_crate::num_traits::Pow;

use crate::gui::windows::settings::Settings;
use crate::inputmap::{InputAction, InputMap};

/// CameraHandler3D is the camera handler for the 3D view
/// It controls the camera using an orbit view
pub struct OrbitCamera {
    pub camera: Camera,
    pub lastscreenpos: Vec2,
    pub last_pos: Option<Vec2>,
    pub targetpos: Vec3,
    pub targetyaw: Radians,
    pub targetpitch: Radians,
    pub targetdist: f32,
    pub maxdist: f32,
}

impl OrbitCamera {
    pub fn update(&mut self, ctx: &mut Context) {
        self.camera.update();
        ctx.gfx.set_camera(self.camera);
        let params = ctx.gfx.render_params.value_mut();
        params.cam_pos = self.camera.eye();
        params.cam_dir = -self.camera.dir();
    }

    pub fn height(&self) -> f32 {
        self.camera.offset().z
    }

    pub fn cull_tess(&self, tess: &mut Tesselator) {
        let p = self.camera.pos;
        tess.cull_rect = Some(AABB::centered(p.xy(), Vec2::splat(4000.0)));
        tess.zoom = 1000.0 / self.height().max(1.0);
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
        let r = self.camera.unproj_ray(pos)?;

        let p = Plane { n: Vec3::Z, o: 0.0 };

        let p = r.intersection_plane(&p)?.xy();
        Some(p.z(height(p)?))
    }

    /// Project a 3D point to the screen
    /// Returns the screen position [0, 1] and the depth
    pub fn project(&self, pos: Vec3) -> (Vec2, f32) {
        self.camera.project(pos)
    }

    fn save(&self) {
        let cam = self.camera;
        rayon::spawn(move || {
            common::saveload::JSONPretty::save_silent(&cam, "camera3D");
        });
    }

    pub fn load(viewport: (u32, u32)) -> Self {
        let camera = common::saveload::JSON::load("camera3D").unwrap_or_else(|_| {
            Camera::new(
                Vec3::new(6511.0, 9590.0, 0.0),
                viewport.0 as f32,
                viewport.1 as f32,
            )
        });

        Self {
            camera,
            lastscreenpos: Default::default(),
            last_pos: Default::default(),
            targetpos: camera.pos,
            targetyaw: camera.yaw,
            targetpitch: camera.pitch,
            targetdist: camera.dist,
            maxdist: 1500.0,
        }
    }

    pub fn camera_movement(
        &mut self,
        ctx: &mut Context,
        delta: f32,
        inps: &InputMap,
        settings: &Settings,
        map_bounds: AABB,
        height: impl Fn(Vec2) -> Option<f32>,
    ) {
        // edge cases (NaN, inf, etc)
        if !self.camera.pos.is_finite() {
            self.camera.pos = Vec3::ZERO;
        }
        if !self.camera.dist.is_finite() {
            self.camera.dist = self.maxdist;
        }
        if !self.camera.yaw.0.is_finite() {
            self.camera.yaw.0 = 0.3;
        }
        if !self.camera.pitch.0.is_finite() {
            self.camera.pitch.0 = 0.3;
        }

        self.save();

        // prepare useful variables
        let delta = delta.min(0.1);
        let off = self.camera.offset();
        let d = off.xy().try_normalize().unwrap_or(Vec2::ZERO) * self.camera.dist;
        let screenpos = ctx.input.mouse.screen;

        // handle inputs
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
            self.targetdist *= (1.0f32 / 1.05).pow(0.5 + 0.1 * inps.wheel.abs());
        }

        if inps.act.contains(&InputAction::Dezoom) {
            self.targetdist *= 1.05f32.pow(0.5 + 0.1 * inps.wheel.abs());
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
            self.targetyaw -= Radians(delta_mouse.x / 100.0);
            self.targetpitch += Radians(delta_mouse.y / 100.0);
            self.targetpitch = self
                .targetpitch
                .min(Radians::HALFPI - Radians(0.01))
                .max(Radians(0.01));
        } else if inps.act.contains(&InputAction::CameraMove) {
            if let Some((last_pos, unprojected)) = self.last_pos.zip(unprojected) {
                self.targetpos += (last_pos - unprojected.xy())
                    .cap_magnitude(50.0 * delta * self.camera.eye().z)
                    .z0();
            }
            self.last_pos = unprojected.map(Vec3::xy);
        }

        // make sure things are in reasonable bounds
        self.targetdist = self.targetdist.clamp(5.0, self.maxdist);
        self.camera.fovy = settings.camera_fov.clamp(1.0, 179.0);
        self.targetpos.x = self.targetpos.x.clamp(map_bounds.ll.x, map_bounds.ur.x);
        self.targetpos.y = self.targetpos.y.clamp(map_bounds.ll.y, map_bounds.ur.y);
        self.targetpos.z = self.targetpos.z.clamp(0.0, 100000.0);

        // smooth camera movement
        if settings.camera_smooth {
            macro_rules! lerpp {
                ($a:expr, $b:expr, $amt:expr, $c:expr) => {
                    let coeff = delta * settings.camera_smooth_tightness * $amt;
                    let diff = $b - $a;
                    if coeff.abs() > 1.0 || $c(diff) < 0.002 {
                        $a = $b;
                    } else {
                        $a += diff * coeff;
                    }
                };
            }

            lerpp!(self.camera.pos, self.targetpos, 8.0, |v: Vec3| v.mag2());
            lerpp!(self.camera.yaw, self.targetyaw, 16.0, |x: Radians| x
                .0
                .abs());
            lerpp!(self.camera.pitch, self.targetpitch, 8.0, |x: Radians| x
                .0
                .abs());
            lerpp!(self.camera.dist, self.targetdist, 8.0, |x: f32| x.abs());
            if (self.targetdist / self.camera.dist - 1.0).abs() < 0.002 {
                self.camera.dist = self.targetdist;
            }
        } else {
            self.camera.pos = self.targetpos;
            self.camera.yaw = self.targetyaw;
            self.camera.pitch = self.targetpitch;
            self.camera.dist = self.targetdist;
        }

        // update orbit center to be height aware
        self.camera.pos.z = height(self.camera.pos.xy())
            .unwrap_or(self.camera.pos.z)
            .clamp(0.0, 100000.0);

        // update camera
        self.update(ctx);
        self.save();
    }
}
