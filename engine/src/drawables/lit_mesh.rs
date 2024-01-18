use std::sync::Arc;

use wgpu::{BindGroupLayout, Device, IndexFormat, RenderPass, RenderPipeline, VertexBufferLayout};

use geom::{Matrix4, Sphere};

use crate::meshbuild::MeshLod;
use crate::{
    CompiledModule, Drawable, GfxContext, Material, MeshInstance, MeshVertex, PipelineBuilder,
    RenderParams, Texture, Uniform, TL,
};

#[derive(Clone)]
pub struct Mesh {
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub lods: Box<[MeshLod]>,
    pub skip_depth: bool,
}

impl Mesh {
    pub(crate) fn lod_select(&self, gfx: &GfxContext) -> Option<&MeshLod> {
        self.lods.iter().find(|x| x.passes_culling(gfx))
    }
}

/// Returns the screen area of a sphere between [0..1] where 1 is the entire screen (if the sphere fits within the screen)
pub fn screen_coverage(gfx: &GfxContext, s: Sphere) -> f32 {
    let v = &gfx.render_params.value().proj;
    let proj_center = v * s.center.w(1.0);
    let proj_center_side =
        v * (s.center + s.radius * gfx.render_params.value().cam_dir.perp_up()).w(1.0);

    let proj_center = proj_center.xyz() / proj_center.w;
    let proj_center_side = proj_center_side.xyz() / proj_center_side.w;

    let proj_radius2 = (proj_center_side - proj_center).mag2();

    proj_radius2 * std::f32::consts::PI
}

#[derive(Clone, Copy, Hash)]
pub(crate) struct MeshPipeline {
    pub(crate) instanced: bool,
    pub(crate) alpha: bool,
    pub(crate) smap: bool,
    pub(crate) depth: bool,
}

const VB_INSTANCED: &[VertexBufferLayout] = &[MeshVertex::desc(), MeshInstance::desc()];
const VB: &[VertexBufferLayout] = &[MeshVertex::desc()];

impl PipelineBuilder for MeshPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str) -> CompiledModule,
    ) -> RenderPipeline {
        let vert = if self.instanced {
            mk_module("instanced_mesh.vert")
        } else {
            mk_module("lit_mesh.vert")
        };

        let vb: &[VertexBufferLayout] = if self.instanced { VB_INSTANCED } else { VB };

        if !self.depth {
            let frag = mk_module("pixel.frag");
            return gfx.color_pipeline(
                "lit_mesh",
                &[
                    &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                    &Material::bindgroup_layout(&gfx.device),
                    &bg_layout_litmesh(&gfx.device),
                ],
                vb,
                &vert,
                &frag,
            );
        }

        if !self.alpha {
            return gfx.depth_pipeline(vb, &vert, None, self.smap);
        }

        let frag = mk_module("alpha_discard.frag");
        gfx.depth_pipeline_bglayout(
            vb,
            &vert,
            Some(&frag),
            self.smap,
            &[
                &gfx.render_params.layout,
                &Material::bindgroup_layout(&gfx.device),
            ],
        )
    }
}

impl Drawable for Mesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let Some(lod) = self.lod_select(gfx) else {
            return;
        };

        rp.set_bind_group(2, &gfx.simplelit_bg, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, index_range) in &lod.primitives {
            let mat = gfx.material(*mat);
            rp.set_pipeline(gfx.get_pipeline(MeshPipeline {
                instanced: false,
                alpha: false,
                smap: false,
                depth: false,
            }));
            rp.set_bind_group(1, &mat.bg, &[]);
            rp.draw_indexed(index_range.clone(), 0, 0..1);

            gfx.perf.drawcall((index_range.end - index_range.start) / 3);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        if self.skip_depth {
            return;
        }
        let Some(lod) = self.lod_select(gfx) else {
            return;
        };
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, index_range) in &lod.primitives {
            let mat = gfx.material(*mat);
            rp.set_pipeline(gfx.get_pipeline(MeshPipeline {
                instanced: false,
                alpha: mat.transparent,
                smap: shadow_cascade.is_some(),
                depth: true,
            }));

            if mat.transparent {
                rp.set_bind_group(1, &mat.bg, &[]);
            }
            rp.draw_indexed(index_range.clone(), 0, 0..1);

            gfx.perf.depth_drawcall(
                (index_range.end - index_range.start) / 3,
                shadow_cascade.is_some(),
            );
        }
    }
}

pub struct LitMeshDepth;
pub struct LitMeshDepthSMap;

pub fn bg_layout_litmesh(device: &Device) -> BindGroupLayout {
    Texture::bindgroup_layout(
        device,
        [
            TL::Float,
            TL::Float,
            TL::DepthArray,
            TL::Cube,
            TL::Cube,
            TL::Float,
            TL::UInt,
            TL::UInt,
        ],
    )
}
