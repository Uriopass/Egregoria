use engine::{Context, MouseButton};
use geom::{Camera, Radians, Vec2, Vec3};

pub struct OrbitCamera {
    pub camera: Camera,
    pub targetpos: Vec3,
    pub targetyaw: Radians,
    pub targetpitch: Radians,
    pub targetdist: f32,
}

impl OrbitCamera {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(Vec3::ZERO, 1920.0, 1080.0),
            targetpos: Default::default(),
            targetyaw: Default::default(),
            targetpitch: Default::default(),
            targetdist: 100.0,
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.camera.update();
        ctx.gfx.set_camera(self.camera);

        let params = ctx.gfx.render_params.value_mut();
        params.cam_pos = self.camera.eye();
        params.cam_dir = -self.camera.dir();
    }

    pub fn resize(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.camera.set_viewport(width, height);
        self.update(ctx);
    }

    #[allow(clippy::redundant_closure_call)]
    pub fn camera_movement(&mut self, ctx: &mut Context) {
        if !self.camera.pos.is_finite() {
            self.camera.pos = Vec3::ZERO;
        }
        if !self.camera.dist.is_finite() {
            self.camera.dist = 1000.0;
        }
        if !self.camera.yaw.0.is_finite() {
            self.camera.yaw.0 = 0.3;
        }
        if !self.camera.pitch.0.is_finite() {
            self.camera.pitch.0 = 0.3;
        }

        let delta = ctx.delta.min(0.1);
        let off = self.camera.offset();
        let d = off.xy().try_normalize().unwrap_or(Vec2::ZERO) * self.camera.dist;

        // D
        if ctx.input.keyboard.pressed_scancode.contains(&32) {
            self.targetpos += -delta * d.perpendicular().z0();
        }
        // A
        if ctx.input.keyboard.pressed_scancode.contains(&30) {
            self.targetpos += delta * d.perpendicular().z0();
        }

        // W
        if ctx.input.keyboard.pressed_scancode.contains(&17) {
            self.targetpos += -delta * d.z0();
        }
        // S
        if ctx.input.keyboard.pressed_scancode.contains(&31) {
            self.targetpos += delta * d.z0();
        }

        if ctx.input.mouse.wheel_delta > 0.0 {
            self.targetdist *= (1.0f32 / 1.05).powf(0.5 + 0.1 * ctx.input.mouse.wheel_delta.abs());
        }

        if ctx.input.mouse.wheel_delta < 0.0 {
            self.targetdist *= 1.05f32.powf(0.5 + 0.1 * ctx.input.mouse.wheel_delta.abs());
        }
        self.targetdist = self.targetdist.clamp(5.0, 100000.0);

        let delta_mouse = ctx.input.mouse.screen_delta;

        if ctx.input.mouse.pressed.contains(&MouseButton::Right) {
            self.targetyaw -= Radians(delta_mouse.x / 100.0);
            self.targetpitch += Radians(delta_mouse.y / 100.0);
            self.targetpitch = self
                .targetpitch
                .min(Radians::HALFPI - Radians(0.01))
                .max(-Radians::HALFPI + Radians(0.01));
        }

        macro_rules! lerpp {
            ($a:expr, $b:expr, $amt:expr, $c:expr) => {
                let coeff = delta * 1.0 * $amt;
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

        self.camera.fovy = 60.0;

        self.update(ctx);
    }
}
