use std::sync::Arc;

use wgpu::{
    BindGroupLayout, Device, IndexFormat, MapMode, RenderPass, RenderPipeline, TextureFormat,
    TextureUsages, VertexBufferLayout,
};

use geom::{Camera, LinearColor, Matrix4, Sphere, Vec2, Vec3};

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
    pub(crate) format: Option<TextureFormat>,
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

            return PipelineBuilder::color(
                "lit_mesh",
                &[
                    &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                    &bg_layout_litmesh(&gfx.device),
                    &Material::bindgroup_layout(&gfx.device),
                ],
                vb,
                &vert,
                &frag,
                self.format.unwrap_or(gfx.sc_desc.format),
            )
            .with_samples(gfx.samples)
            .build(&gfx.device);
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

impl Mesh {
    pub fn render_to_texture(&self, cam: &Camera, gfx: &GfxContext, dest: &Texture) {}

    pub fn render_to_image(
        &self,
        cam: &Camera,
        gfx: &GfxContext,
        size: u32,
        on_complete: impl FnOnce(image::RgbaImage) + Send + 'static,
    ) {
        let target_image = TextureBuilder::empty(size, size, 1, TextureFormat::Bgra8UnormSrgb)
            .with_usage(TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC)
            .with_label("mesh_render_to_image")
            .build_no_queue(&gfx.device);

        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("mesh_render_to_image"),
            });

        let mut params = gfx.render_params.clone(&gfx.device);
        let value = params.value_mut();
        value.proj = cam.proj_cache;
        value.inv_proj = cam.inv_proj_cache;
        value.cam_dir = cam.dir();
        value.cam_pos = cam.eye();
        value.viewport = Vec2::splat(size as f32);
        value.sun = Vec3::new(1.0, 1.0, 1.0).normalize();
        value.sun_col = LinearColor::WHITE;
        value.time = 0.0;
        value.time_always = 0.0;
        value.shadow_mapping_resolution = 0;

        params.upload_to_gpu(&gfx.queue);

        let mut cpy = self.clone();
        let mut lod_cpy = cpy.lods[0].clone();
        lod_cpy.screen_coverage = f32::NEG_INFINITY;
        lod_cpy.bounding_sphere = Sphere::new(Vec3::ZERO, 10000.0);
        cpy.lods = vec![lod_cpy].into_boxed_slice();

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mesh_render_to_image"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_image.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &gfx.fbos.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rp.set_bind_group(0, &params.bg, &[]);

            cpy.draw(&gfx, &mut rp);
        }

        let image_data_buf = Arc::new(gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("mesh_render_to_image"),
            size: (4 * size * size) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &target_image.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &image_data_buf,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * size), // 4 bytes per pixel * 4 pixels per row
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
        );

        gfx.queue.submit(std::iter::once(encoder.finish()));

        let image_data_buf_cpy = image_data_buf.clone();
        image_data_buf.slice(..).map_async(MapMode::Read, move |v| {
            if v.is_err() {
                log::error!("Failed to map buffer for reading for render_to_image");
                return;
            }

            let v = image_data_buf_cpy.slice(..).get_mapped_range();

            let Some(rgba) = image::RgbaImage::from_raw(size, size, v.to_vec()) else {
                log::error!("Failed to create image from buffer for render_to_image");
                return;
            };

            on_complete(rgba);
        });
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
                format: None,
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
                format: None,
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
