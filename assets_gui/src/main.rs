use std::path::Path;

use egui::FontFamily::{Monospace, Proportional};
use egui::FontId;

use engine::meshload::load_mesh_with_properties;
use engine::{Context, FrameContext, GfxContext, SpriteBatchBuilder};
use geom::{vec3, InfiniteFrustrum, LinearColor, Plane, Vec2, Vec3};

use crate::gui::{Gui, Inspected, Shown};
use crate::orbit_camera::OrbitCamera;

mod companies;
mod gui;
mod orbit_camera;

struct State {
    gui: Gui,
    camera: OrbitCamera,
    last_inspect: Inspected,
}

impl engine::framework::State for State {
    fn new(ctx: &mut Context) -> Self {
        let gfx = &mut ctx.gfx;

        gfx.render_params.value_mut().shadow_mapping_resolution = 2048;
        gfx.sun_shadowmap = GfxContext::mk_shadowmap(&gfx.device, 2048);
        gfx.update_simplelit_bg();

        let mut style = (*ctx.egui.egui.style()).clone();

        style.text_styles = [
            (egui::TextStyle::Small, FontId::new(15.0, Proportional)),
            (egui::TextStyle::Body, FontId::new(18.5, Proportional)),
            (egui::TextStyle::Button, FontId::new(18.5, Proportional)),
            (egui::TextStyle::Heading, FontId::new(25.0, Proportional)),
            (egui::TextStyle::Monospace, FontId::new(18.0, Monospace)),
        ]
        .into();

        ctx.egui.egui.set_style(style);

        let camera = OrbitCamera::new();

        Self {
            camera,
            gui: Gui::new(),
            last_inspect: Inspected::None,
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.camera.camera_movement(ctx);

        if self.gui.inspected != self.last_inspect {
            self.last_inspect = self.gui.inspected;
            self.gui.shown = create_shown(&mut ctx.gfx, self, self.gui.inspected);
        }

        let gfx = &mut ctx.gfx;

        let params = gfx.render_params.value_mut();

        let sun = vec3(1.0, -1.0, 1.0).normalize();
        params.time_always = (params.time_always + ctx.delta) % 3600.0;
        params.sun_col = sun.z.max(0.0).sqrt().sqrt()
            * LinearColor::new(1.0, 0.95 + sun.z * 0.05, 0.95 + sun.z * 0.05, 1.0);
        params.sun = sun;
        params.viewport = Vec2::new(gfx.size.0 as f32, gfx.size.1 as f32);
        params.shadow_mapping_resolution = 2048;
        params.sun_shadow_proj = self
            .camera
            .camera
            .build_sun_shadowmap_matrix(
                sun,
                params.shadow_mapping_resolution as f32,
                &InfiniteFrustrum::new([Plane::X; 5]),
            )
            .try_into()
            .unwrap();
    }

    fn render(&mut self, fc: &mut FrameContext) {
        fc.draw(self.gui.shown.clone());
    }

    fn resized(&mut self, ctx: &mut Context, size: (u32, u32, f64)) {
        self.camera.resize(ctx, size.0 as f32, size.1 as f32);
    }

    fn render_gui(&mut self, ui: &egui::Context) {
        self.gui(ui);
    }
}

fn create_shown(gfx: &mut GfxContext, state: &State, inspected: Inspected) -> Shown {
    match inspected {
        Inspected::None => Shown::None,
        Inspected::Company(i) => {
            let comp = &state.gui.companies.companies[i];
            let p = Path::new(&comp.asset_location);
            match p.extension() {
                Some(x) if (x == "png" || x == "jpg") => {
                    let tex = match gfx.try_texture(p, "sprite texture") {
                        Ok(x) => x,
                        Err(e) => {
                            return Shown::Error(format!(
                                "could not load texture {}: {}",
                                comp.asset_location, e
                            ))
                        }
                    };
                    let mut sb: SpriteBatchBuilder<false> = SpriteBatchBuilder::new(tex, gfx);
                    sb.push(Vec3::ZERO, Vec3::X, LinearColor::WHITE, (100.0, 100.0));
                    Shown::Sprite(sb.build(gfx).unwrap())
                }
                Some(x) if x == "glb" => {
                    let model = match load_mesh_with_properties(gfx, &comp.asset_location) {
                        Ok(x) => x,
                        Err(e) => {
                            return Shown::Error(format!(
                                "could not load model {}:\n{:?}",
                                comp.asset_location, e
                            ))
                        }
                    };

                    Shown::Model(model)
                }
                Some(_) => Shown::Error(format!(
                    "unknown asset type for path: {}",
                    comp.asset_location
                )),
                None => Shown::Error(format!("no extension for path: {}", comp.asset_location)),
            }
        }
    }
}

fn main() {
    common::logger::MyLog::init();

    engine::framework::start::<State>();
}
