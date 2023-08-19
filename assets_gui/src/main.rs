use geom::{vec3, Camera, InfiniteFrustrum, LinearColor, Matrix4, Plane, Radians, Vec2, Vec3};
use wgpu_engine::{Context, FrameContext, GfxContext, KeyCode, MouseButton};

struct State {
    is_captured: bool,

    camera: Camera,
}

impl wgpu_engine::framework::State for State {
    fn new(ctx: &mut Context) -> Self {
        let gfx = &mut ctx.gfx;

        gfx.render_params.value_mut().shadow_mapping_resolution = 2048;
        gfx.sun_shadowmap = GfxContext::mk_shadowmap(&gfx.device, 2048);
        gfx.update_simplelit_bg();

        let mut camera = Camera::new(vec3(9.0, -30.0, 13.0), 1000.0, 1000.0);
        camera.dist = 0.0;
        camera.pitch = Radians(0.0);
        camera.yaw = Radians(-std::f32::consts::PI / 2.0);

        Self {
            camera,
            is_captured: false,
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        if ctx.input.mouse.pressed.contains(&MouseButton::Left) {
            let _ = ctx
                .window
                .set_cursor_grab(wgpu_engine::CursorGrabMode::Confined);
            ctx.window.set_cursor_visible(false);
            self.is_captured = true;
        }

        if ctx.input.cursor_left {
            let _ = ctx
                .window
                .set_cursor_grab(wgpu_engine::CursorGrabMode::None);
            ctx.window.set_cursor_visible(true);
            self.is_captured = false;
        }

        if ctx.input.keyboard.pressed.contains(&KeyCode::Escape) {
            let _ = ctx
                .window
                .set_cursor_grab(wgpu_engine::CursorGrabMode::None);
            ctx.window.set_cursor_visible(true);
            self.is_captured = false;
        }

        let delta = ctx.delta;
        let cam_speed = if ctx.input.keyboard.pressed_scancode.contains(&42) {
            3.0
        } else {
            30.0
        } * delta;

        if ctx.input.keyboard.pressed_scancode.contains(&17) {
            self.camera.pos -= self
                .camera
                .dir()
                .xy()
                .z0()
                .try_normalize()
                .unwrap_or(Vec3::ZERO)
                * cam_speed;
        }
        if ctx.input.keyboard.pressed_scancode.contains(&31) {
            self.camera.pos += self
                .camera
                .dir()
                .xy()
                .z0()
                .try_normalize()
                .unwrap_or(Vec3::ZERO)
                * cam_speed;
        }
        if ctx.input.keyboard.pressed_scancode.contains(&30) {
            self.camera.pos += self
                .camera
                .dir()
                .perp_up()
                .try_normalize()
                .unwrap_or(Vec3::ZERO)
                * cam_speed;
        }
        if ctx.input.keyboard.pressed_scancode.contains(&32) {
            self.camera.pos -= self
                .camera
                .dir()
                .perp_up()
                .try_normalize()
                .unwrap_or(Vec3::ZERO)
                * cam_speed;
        }
        if ctx.input.keyboard.pressed_scancode.contains(&57) {
            self.camera.pos += vec3(0.0, 0.0, 1.0) * cam_speed;
        }
        if ctx.input.keyboard.pressed_scancode.contains(&29) {
            self.camera.pos -= vec3(0.0, 0.0, 1.0) * cam_speed;
        }

        if self.is_captured {
            let delta = ctx.input.mouse.screen_delta;

            self.camera.yaw.0 -= 0.001 * delta.x;
            self.camera.pitch.0 += 0.001 * delta.y;
            self.camera.pitch.0 = self.camera.pitch.0.clamp(-1.5, 1.5);
        }

        let sun = vec3(1.0, -1.0, 1.0).normalize();

        let gfx = &mut ctx.gfx;

        let viewproj = self.camera.build_view_projection_matrix();
        let inv_viewproj = viewproj.invert().unwrap_or_else(Matrix4::zero);
        gfx.set_proj(viewproj);
        gfx.set_inv_proj(inv_viewproj);

        let params = gfx.render_params.value_mut();
        params.time_always = (params.time_always + delta) % 3600.0;
        params.sun_col = sun.z.max(0.0).sqrt().sqrt()
            * LinearColor::new(1.0, 0.95 + sun.z * 0.05, 0.95 + sun.z * 0.05, 1.0);
        params.cam_pos = self.camera.eye();
        params.cam_dir = self.camera.dir();
        params.sun = sun;
        params.viewport = Vec2::new(gfx.size.0 as f32, gfx.size.1 as f32);
        self.camera.dist = 300.0;
        params.sun_shadow_proj = self
            .camera
            .build_sun_shadowmap_matrix(
                sun,
                params.shadow_mapping_resolution as f32,
                &InfiniteFrustrum::new([Plane::X; 5]),
            )
            .try_into()
            .unwrap();
        self.camera.dist = 0.0;
        params.shadow_mapping_resolution = 2048;
    }

    fn render(&mut self, _fc: &mut FrameContext) {}

    fn resized(&mut self, _ctx: &mut Context, size: (u32, u32)) {
        self.camera.set_viewport(size.0 as f32, size.1 as f32);
    }
}

fn main() {
    common::logger::MyLog::init();

    wgpu_engine::framework::start::<State>();
}
