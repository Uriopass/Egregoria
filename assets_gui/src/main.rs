use crate::lod::{export_doc_opt, lod_generate, LodGenerateParams};
use common::unwrap_cont;
use engine::meshload::load_mesh_with_properties;
use engine::{
    Context, FrameContext, GfxContext, GfxSettings, InstancedMeshBuilder, MeshInstance,
    SpriteBatchBuilder,
};
use geom::{vec3, InfiniteFrustrum, LinearColor, Plane, Vec2, Vec3};
use prototypes::{try_prototype, RenderAsset};
use std::path::PathBuf;

use crate::orbit_camera::OrbitCamera;
use crate::yakui_gui::{Gui, Inspected, Shown};

mod lod;
mod orbit_camera;
mod yakui_gui;

#[derive(Debug)]
pub enum GUIAction {
    GenerateLOD(PathBuf, LodGenerateParams),
}

struct State {
    camera: OrbitCamera,
    last_inspect: Inspected,
    gui: Gui,
    actions: Vec<GUIAction>,
}

impl engine::framework::State for State {
    fn new(ctx: &mut Context) -> Self {
        goryak::set_blur_texture(ctx.yakui.blur_bg_texture);

        let gfx = &mut ctx.gfx;

        gfx.render_params.value_mut().shadow_mapping_resolution = 2048;
        gfx.sun_shadowmap = GfxContext::mk_shadowmap(&gfx.device, 2048);
        gfx.update_simplelit_bg();

        let camera = OrbitCamera::new();

        let mut gui = Gui::new();

        gui.inspected = Inspected::None;

        Self {
            camera,
            last_inspect: Inspected::None,
            gui,
            actions: vec![],
        }
    }

    fn update(&mut self, ctx: &mut Context) {
        self.camera.camera_movement(ctx);

        for action in std::mem::take(&mut self.actions) {
            self.do_action(ctx, action);
        }

        if self.gui.inspected != self.last_inspect {
            self.last_inspect = self.gui.inspected;
            self.gui.shown = create_shown(&mut ctx.gfx, self, self.gui.inspected);
        }

        let gfx = &mut ctx.gfx;

        let params = gfx.render_params.value_mut();

        let sun = vec3(1.0, -1.0, 1.0).normalize();
        params.time_always = (params.time_always + ctx.delta) % 3600.0;
        params.sun_col = 4.0
            * sun.z.max(0.0).sqrt().sqrt()
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

        gfx.update_settings(GfxSettings::default());
    }

    fn render(&mut self, fc: &mut FrameContext) {
        fc.draw(self.gui.shown.clone());
    }

    fn resized(&mut self, ctx: &mut Context, size: (u32, u32, f64)) {
        self.camera.resize(ctx, size.0 as f32, size.1 as f32);
    }

    fn render_yakui(&mut self) {
        self.gui_yakui();
    }
}

impl State {
    fn do_action(&mut self, ctx: &mut Context, action: GUIAction) {
        log::info!("{:?}", action);
        let gfx = &mut ctx.gfx;
        match action {
            GUIAction::GenerateLOD(ref path, params) => {
                let Ok((_, mut cpumesh)) = load_mesh_with_properties(gfx, path, true) else {
                    return;
                };

                if let Err(e) = lod_generate(&mut cpumesh, params) {
                    log::error!("{:?}", e);
                }
                export_doc_opt(&cpumesh);

                self.last_inspect = Inspected::None;
            }
        }
    }
}

fn create_shown(gfx: &mut GfxContext, _state: &State, inspected: Inspected) -> Shown {
    match inspected {
        Inspected::None => Shown::None,
        Inspected::Company(i) => {
            let comp = try_prototype(i).unwrap();
            match comp.asset {
                RenderAsset::Sprite { ref path } => {
                    let tex = match gfx.try_texture(path, "sprite texture") {
                        Ok(x) => x,
                        Err(e) => {
                            return Shown::Error(format!(
                                "could not load texture {}: {}",
                                comp.asset, e
                            ))
                        }
                    };
                    let mut sb: SpriteBatchBuilder<false> = SpriteBatchBuilder::new(&tex, gfx);
                    sb.push(Vec3::ZERO, Vec3::X, LinearColor::WHITE, (100.0, 100.0));
                    Shown::Sprite(sb.build(gfx).unwrap())
                }
                RenderAsset::Mesh { ref path } => {
                    let (mesh, cpu) = match load_mesh_with_properties(gfx, path, false) {
                        Ok(x) => x,
                        Err(e) => {
                            return Shown::Error(format!(
                                "could not load model {}:\n{:?}",
                                comp.asset, e
                            ))
                        }
                    };
                    let size = mesh.lods[0].bounding_sphere.radius;
                    let mut meshes = vec![];
                    for (i, mut lod) in mesh.lods.iter().cloned().enumerate() {
                        let mut cpy = mesh.clone();
                        lod.screen_coverage = 0.0;
                        cpy.lods = vec![lod].into_boxed_slice();

                        let mut b: InstancedMeshBuilder<false> = InstancedMeshBuilder::new(cpy);
                        b.instances.push(MeshInstance {
                            pos: Vec3::x(i as f32 * size * 2.0),
                            dir: Vec3::X,
                            tint: LinearColor::WHITE,
                        });

                        meshes.push(unwrap_cont!(b.build(gfx)));
                    }

                    Shown::Model((mesh, meshes, cpu))
                }
            }
        }
    }
}

fn main() {
    engine::framework::init();
    unsafe {
        prototypes::load_prototypes("./").unwrap();
    }
    engine::framework::start::<State>();
}
