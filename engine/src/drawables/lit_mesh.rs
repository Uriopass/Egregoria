use std::sync::Arc;

use wgpu::{
    BindGroupLayout, Device, IndexFormat, RenderPass, RenderPipeline, TextureFormat,
    TextureViewDescriptor, TextureViewDimension, VertexBufferLayout,
};

use geom::{Camera, InfiniteFrustrum, LinearColor, Matrix4, Sphere, Vec2, Vec3};

use crate::meshbuild::MeshLod;
use crate::{
    CompiledModule, Drawable, GfxContext, Material, MeshInstance, MeshVertex, PipelineBuilder,
    PipelineKey, RenderParams, Texture, TextureBuilder, Uniform, TL,
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
    pub(crate) offscreen_render: bool,
    pub(crate) instanced: bool,
    pub(crate) alpha: bool,
    pub(crate) smap: bool,
    pub(crate) depth: bool,
}

const VB_INSTANCED: &[VertexBufferLayout] = &[MeshVertex::desc(), MeshInstance::desc()];
const VB: &[VertexBufferLayout] = &[MeshVertex::desc()];

impl PipelineKey for MeshPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str, &[&str]) -> CompiledModule,
    ) -> RenderPipeline {
        let vert = if self.instanced {
            mk_module("instanced_mesh.vert", &[])
        } else {
            mk_module("lit_mesh.vert", &[])
        };

        let vb: &[VertexBufferLayout] = if self.instanced { VB_INSTANCED } else { VB };

        if !self.depth {
            let extra_defines = if self.offscreen_render {
                &["OFFSCREEN_RENDER"] as &[&str]
            } else {
                &[]
            };

            let frag = mk_module("pixel.frag", extra_defines);

            let bglayout = match self.offscreen_render {
                true => bg_layout_offscreen_render(&gfx.device),
                false => bg_layout_litmesh(&gfx.device),
            };

            let layouts = [
                &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                &bglayout,
                &Material::bindgroup_layout(&gfx.device),
            ];

            let mut builder = PipelineBuilder::color(
                "lit_mesh",
                &layouts,
                vb,
                &vert,
                &frag,
                match self.offscreen_render {
                    true => TextureFormat::Rgba8UnormSrgb,
                    false => gfx.sc_desc.format,
                },
            )
            .with_samples(match self.offscreen_render {
                true => 4,
                false => gfx.samples,
            });

            if self.offscreen_render {
                builder = builder.with_depth_write();
            }

            return builder.build(&gfx.device);
        }

        if !self.alpha {
            return gfx.depth_pipeline(vb, &vert, None, self.smap);
        }

        let frag = mk_module("alpha_discard.frag", &[]);
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

impl Mesh {
    pub fn render_to_texture(
        &self,
        cam: &Camera,
        gfx: &GfxContext,
        dest: &Texture,
        dest_msaa: &Texture,
    ) {
        const SHADOWMAP_RES: i32 = 1024;

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("mesh_render_to_image"),
            });

        let sun = Vec3::new(0.5, -1.3, 0.9).normalize();

        let smap_mat =
            cam.build_sun_shadowmap_matrix(sun, SHADOWMAP_RES as f32, &InfiniteFrustrum::EMPTY);

        let mut params = gfx.render_params.clone(&gfx.device);
        let value = params.value_mut();
        value.proj = cam.proj_cache;
        value.inv_proj = cam.inv_proj_cache;
        value.cam_dir = cam.dir();
        value.cam_pos = cam.eye();
        value.viewport = Vec2::new(dest.extent.width as f32, dest.extent.height as f32);
        value.sun = sun;
        value.sun_col = 3.5 * LinearColor::new(1.0, 0.95, 1.0, 1.0);
        value.sun_shadow_proj = [
            smap_mat[0],
            Matrix4::zero(),
            Matrix4::zero(),
            Matrix4::zero(),
        ];
        value.time = 0.0;
        value.time_always = 0.0;
        value.shadow_mapping_resolution = SHADOWMAP_RES;

        params.upload_to_gpu(&gfx.queue);

        let depth = TextureBuilder::empty(
            dest.extent.width,
            dest.extent.height,
            1,
            TextureFormat::Depth32Float,
        )
        .with_sample_count(4)
        .build_no_queue(&gfx.device);

        let mut smap = TextureBuilder::empty(1024, 1024, 1, TextureFormat::Depth32Float)
            .build_no_queue(&gfx.device);

        let mut params_smap = params.clone(&gfx.device);
        let value_smap = params_smap.value_mut();
        value_smap.proj = smap_mat[0];
        params_smap.upload_to_gpu(&gfx.queue);

        let smap_view = smap.mip_view(0);

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mesh_render_to_image_shadow_map"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &smap_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_bind_group(0, &params_smap.bg, &[]);

            rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

            for (mat, index_range) in &self.lods[0].primitives {
                let mat = gfx.material(*mat);
                rp.set_pipeline(gfx.get_pipeline(MeshPipeline {
                    offscreen_render: true,
                    instanced: false,
                    alpha: mat.transparent,
                    smap: true,
                    depth: true,
                }));

                if mat.transparent {
                    rp.set_bind_group(1, &mat.bg, &[]);
                }
                rp.draw_indexed(index_range.clone(), 0, 0..1);
            }
        }

        smap.view = smap.texture.create_view(&TextureViewDescriptor {
            dimension: Some(TextureViewDimension::D2Array),
            ..Default::default()
        });
        smap.sampler = gfx.device.create_sampler(&Texture::depth_compare_sampler());

        let simplelit_bg = Texture::multi_bindgroup(
            &[
                &gfx.read_texture("assets/sprites/blue_noise_512.png")
                    .expect("blue noise not initialized"),
                &smap,
                &gfx.pbr.diffuse_irradiance_cube,
                &gfx.pbr.specular_prefilter_cube,
                &gfx.pbr.split_sum_brdf_lut,
            ],
            &gfx.device,
            &bg_layout_offscreen_render(&gfx.device),
        );

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mesh_render_to_image"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &dest_msaa.view,
                    resolve_target: Some(&dest.view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_bind_group(0, &params.bg, &[]);
            rp.set_bind_group(1, &simplelit_bg, &[]);
            rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

            for (mat, index_range) in &self.lods[0].primitives {
                let mat = gfx.material(*mat);
                rp.set_pipeline(gfx.get_pipeline(MeshPipeline {
                    offscreen_render: true,
                    instanced: false,
                    alpha: false,
                    smap: false,
                    depth: false,
                }));
                rp.set_bind_group(2, &mat.bg, &[]);
                rp.draw_indexed(index_range.clone(), 0, 0..1);
            }
        }

        gfx.queue.submit(std::iter::once(encoder.finish()));
    }
}

impl Drawable for Mesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let Some(lod) = self.lod_select(gfx) else {
            return;
        };

        rp.set_bind_group(1, &gfx.simplelit_bg, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

        for (mat, index_range) in &lod.primitives {
            let mat = gfx.material(*mat);
            rp.set_pipeline(gfx.get_pipeline(MeshPipeline {
                offscreen_render: false,
                instanced: false,
                alpha: false,
                smap: false,
                depth: false,
            }));
            rp.set_bind_group(2, &mat.bg, &[]);
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
                offscreen_render: false,
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

pub fn bg_layout_litmesh(device: &Device) -> BindGroupLayout {
    Texture::bindgroup_layout(
        device,
        [
            TL::Float,
            TL::DepthArray,
            TL::Cube,
            TL::Cube,
            TL::Float,
            TL::Float,
            TL::Float,
            TL::UInt,
            TL::UInt,
        ],
    )
}

pub fn bg_layout_offscreen_render(device: &Device) -> BindGroupLayout {
    Texture::bindgroup_layout(
        device,
        [TL::Float, TL::DepthArray, TL::Cube, TL::Cube, TL::Float],
    )
}
