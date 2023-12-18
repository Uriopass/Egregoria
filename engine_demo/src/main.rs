use common::{AudioKind, History};
use engine::meshload::load_mesh;
use engine::{
    Context, FrameContext, GfxSettings, InstancedMeshBuilder, KeyCode, MeshInstance, MouseButton,
    ShadowQuality,
};
use geom::{
    vec3, Camera, Degrees, InfiniteFrustrum, LinearColor, Matrix4, Plane, Radians, Vec2, Vec3,
};

use crate::helmet::Helmet;
use crate::spheres::Spheres;
use crate::terrain::Terrain;

mod helmet;
mod spheres;
mod terrain;

trait DemoElement {
    fn name(&self) -> &'static str;
    fn init(ctx: &mut Context) -> Self
    where
        Self: Sized;
    fn update(&mut self, ctx: &mut Context, cam: &Camera);
    fn render(&mut self, fc: &mut FrameContext, cam: &Camera, frustrum: &InfiniteFrustrum);
    fn render_gui(&mut self, _ui: &mut egui::Ui) {}
}

struct State {
    demo_elements: Vec<(Box<dyn DemoElement>, bool)>,

    is_captured: bool,

    camera: Camera,
    camera_speed: f32,
    frustrum: InfiniteFrustrum,
    last_cam: Camera,

    freeze_cam: bool,

    delta: f32,
    play_queue: Vec<&'static str>,

    ms_hist: History,

    gfx_settings: GfxSettings,
    sun_angle: Degrees,
}

impl engine::framework::State for State {
    fn new(ctx: &mut Context) -> Self {
        let gfx = &mut ctx.gfx;

        let mut meshes = vec![];

        if let Ok(m) = load_mesh(gfx, "DamagedHelmet.glb") {
            let mut i = InstancedMeshBuilder::<true>::new(m);
            i.instances.push(MeshInstance {
                pos: vec3(50.0, 00.0, 0.0),
                dir: Vec3::X,
                tint: LinearColor::WHITE,
            });
            meshes.push(i.build(gfx).unwrap());
        }

        let mut camera = Camera::new(vec3(9.0, -30.0, 13.0), 1000.0, 1000.0);
        camera.dist = 0.0;
        camera.pitch = Radians(0.0);
        camera.yaw = Radians(-std::f32::consts::PI / 2.0);

        ctx.audio.set_settings(100.0, 100.0, 100.0, 100.0);

        let mut gfx_settings = GfxSettings::default();
        gfx_settings.shader_debug = true;

        Self {
            demo_elements: vec![
                (Box::new(Spheres::init(ctx)), true),
                (Box::new(Helmet::init(ctx)), true),
                (Box::new(Terrain::init(ctx)), true),
            ],
            camera,
            is_captured: false,
            delta: 0.0,
            play_queue: vec![],
            camera_speed: 100.0,
            frustrum: InfiniteFrustrum::new([Plane::X; 5]),
            last_cam: camera,
            freeze_cam: false,
            ms_hist: History::new(128),
            gfx_settings,
            sun_angle: Degrees(0.0),
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.delta = ctx.delta;

        ctx.gfx.update_settings(self.gfx_settings);
        self.ms_hist.add_value(ctx.delta);

        let delta = self.camera_movement(ctx);

        let sun = Vec2::from_angle(self.sun_angle.into())
            .z0()
            .rotate_up(vec3(1.0, 0.0, 1.0).normalize())
            .normalize();

        let gfx = &mut ctx.gfx;

        let viewproj = self.camera.build_view_projection_matrix();
        let inv_viewproj = viewproj.invert().unwrap_or_else(Matrix4::zero);
        gfx.set_proj(viewproj);
        gfx.set_inv_proj(inv_viewproj);

        let params = gfx.render_params.value_mut();
        params.time_always = (params.time_always + delta) % 3600.0;
        params.sun_col = 4.0
            * sun.z.max(0.0).sqrt().sqrt()
            * LinearColor::new(1.0, 0.95 + sun.z * 0.05, 0.95 + sun.z * 0.05, 1.0);
        if !self.freeze_cam {
            params.cam_pos = self.camera.eye();
            params.cam_dir = self.camera.dir();
        }
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

        for (de, enabled) in &mut self.demo_elements {
            if !*enabled {
                continue;
            }
            de.update(ctx, &self.camera);
        }

        for v in self.play_queue.drain(..) {
            ctx.audio.play(&v, AudioKind::Ui);
        }
    }

    fn render(&mut self, fc: &mut FrameContext) {
        if !self.freeze_cam {
            self.frustrum = InfiniteFrustrum::from_reversez_invviewproj(
                self.camera.eye(),
                fc.gfx.render_params.value().inv_proj,
            );
            self.last_cam = self.camera;
        }

        for (de, enabled) in &mut self.demo_elements {
            if !*enabled {
                continue;
            }
            de.render(fc, &self.last_cam, &self.frustrum);
        }
    }

    fn resized(&mut self, _: &mut Context, size: (u32, u32, f64)) {
        self.camera.set_viewport(size.0 as f32, size.1 as f32);
    }

    fn render_gui(&mut self, ui: &egui::Context) {
        egui::Window::new("Demo elements")
            .resizable(true)
            .show(ui, |ui| {
                ui.add(egui::Slider::new(&mut self.sun_angle.0, 0.0..=360.0).text("Sun angle"));

                for (de, enabled) in &mut self.demo_elements {
                    ui.checkbox(enabled, de.name());
                    de.render_gui(ui);
                }

                if ui.button("play sound: road_lay").clicked() {
                    self.play_queue.push("road_lay");
                }
            });

        egui::Window::new("Settings")
            .resizable(true)
            .show(ui, |ui| {
                let avg_ms = self.ms_hist.avg();
                ui.label(format!(
                    "Avg (128 frames): {:.1}ms {:.0}FPS",
                    1000.0 * avg_ms,
                    1.0 / avg_ms
                ));

                ui.add(egui::Slider::new(&mut self.camera_speed, 1.0..=100.0).text("Camera speed"));
                ui.checkbox(&mut self.freeze_cam, "Freeze camera");

                ui.checkbox(&mut self.gfx_settings.fullscreen, "Fullscreen");
                ui.checkbox(&mut self.gfx_settings.vsync, "VSync");
                ui.checkbox(&mut self.gfx_settings.fog, "Fog");
                ui.checkbox(&mut self.gfx_settings.ssao, "SSAO");
                ui.checkbox(&mut self.gfx_settings.terrain_grid, "Terrain grid");

                let mut shadows = self.gfx_settings.shadows.size().is_some();
                ui.checkbox(&mut shadows, "Shadows");
                self.gfx_settings.shadows = if shadows {
                    ShadowQuality::High
                } else {
                    ShadowQuality::NoShadows
                };

                ui.checkbox(&mut self.gfx_settings.shader_debug, "Shader debug");
                ui.checkbox(&mut self.gfx_settings.pbr_enabled, "PBR Environment Update");
            });
    }
}

impl State {
    fn camera_movement(&mut self, ctx: &mut Context) -> f32 {
        if ctx.input.mouse.pressed.contains(&MouseButton::Left) {
            let _ = ctx
                .gfx
                .window
                .set_cursor_grab(engine::CursorGrabMode::Confined);
            ctx.gfx.window.set_cursor_visible(false);
            self.is_captured = true;
        }

        if ctx.input.cursor_left {
            let _ = ctx.gfx.window.set_cursor_grab(engine::CursorGrabMode::None);
            ctx.gfx.window.set_cursor_visible(true);
            self.is_captured = false;
        }

        if ctx.input.keyboard.pressed.contains(&KeyCode::Escape) {
            let _ = ctx.gfx.window.set_cursor_grab(engine::CursorGrabMode::None);
            ctx.gfx.window.set_cursor_visible(true);
            self.is_captured = false;
        }

        let delta = ctx.delta;
        let cam_speed = if ctx.input.keyboard.pressed_scancode.contains(&42) {
            3.0
        } else {
            30.0
        } * delta
            * self.camera_speed;

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
            self.camera.pos += vec3(0.0, 0.0, 0.5) * cam_speed;
        }
        if ctx.input.keyboard.pressed_scancode.contains(&29) {
            self.camera.pos -= vec3(0.0, 0.0, 0.5) * cam_speed;
        }

        if self.is_captured {
            let delta = ctx.input.mouse.screen_delta;

            self.camera.yaw.0 -= 0.001 * delta.x;
            self.camera.pitch.0 += 0.001 * delta.y;
            self.camera.pitch.0 = self.camera.pitch.0.clamp(-1.5, 1.5);
        }
        delta
    }
}

fn main() {
    common::logger::MyLog::init();

    engine::framework::start::<State>();
}
