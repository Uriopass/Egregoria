use crate::pbuffer::PBuffer;
use crate::{
    Drawable, GfxContext, IndexType, MaterialID, MeshVertex, RenderParams, Texture, Uniform, VBDesc,
};
use std::sync::Arc;
use wgpu::{BindGroupLayout, BindGroupLayoutEntry, BufferUsages, Device, IndexFormat, RenderPass};

pub struct MeshBuilder {
    pub(crate) vertices: Vec<MeshVertex>,
    pub(crate) indices: Vec<IndexType>,
    pub(crate) vbuffer: PBuffer,
    pub(crate) ibuffer: PBuffer,
    pub(crate) material: MaterialID,
}

impl MeshBuilder {
    pub fn new(material: MaterialID) -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            vbuffer: PBuffer::new(BufferUsages::VERTEX),
            ibuffer: PBuffer::new(BufferUsages::INDEX),
            material,
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn extend(&mut self, vertices: &[MeshVertex], indices: &[IndexType]) -> &mut Self {
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|x| x + offset));
        self
    }

    #[inline(always)]
    pub fn extend_with(&mut self, f: impl FnOnce(&mut Vec<MeshVertex>, &mut dyn FnMut(IndexType))) {
        let offset = self.vertices.len() as IndexType;
        let vertices = &mut self.vertices;
        let indices = &mut self.indices;
        let mut x = move |index: IndexType| {
            indices.push(index + offset);
        };
        f(vertices, &mut x);
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<Mesh> {
        if self.vertices.is_empty() {
            return None;
        }

        self.vbuffer
            .write(gfx, bytemuck::cast_slice(&self.vertices));
        self.ibuffer.write(gfx, bytemuck::cast_slice(&self.indices));

        Some(Mesh {
            vertex_buffer: self.vbuffer.inner()?,
            index_buffer: self.ibuffer.inner()?,
            material: self.material,
            n_indices: self.indices.len() as u32,
            transparent: false,
            skip_depth: false,
            double_sided: false,
        })
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub material: MaterialID,
    pub n_indices: u32,
    pub transparent: bool,
    pub skip_depth: bool,
    pub double_sided: bool,
}

#[derive(Clone, Copy, Hash)]
struct LitMeshPipeline {
    alpha: bool,
    smap: bool,
    depth: bool,
    double_sided: bool,
}

impl Mesh {
    pub fn setup(gfx: &mut GfxContext) {
        for double_sided in [false, true] {
            let pipeline = LitMeshPipeline {
                alpha: false,
                smap: false,
                depth: false,
                double_sided,
            };

            gfx.register_pipeline(
                pipeline,
                &["lit_mesh.vert", "pixel.frag"],
                Box::new(move |m, gfx| {
                    let vert = &m[0];
                    let frag = &m[1];
                    let vb = &[MeshVertex::desc()];
                    gfx.color_pipeline(
                        "lit_mesh",
                        &[
                            &gfx.projection.layout,
                            &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                            &Texture::bindgroup_layout(&gfx.device),
                            &bg_layout_litmesh(&gfx.device),
                        ],
                        vb,
                        vert,
                        frag,
                        0,
                        double_sided,
                    )
                }),
            );

            for smap in [false, true] {
                let pipeline_depth = LitMeshPipeline {
                    alpha: false,
                    smap,
                    depth: true,
                    double_sided,
                };
                gfx.register_pipeline(
                    pipeline_depth,
                    &["lit_mesh.vert"],
                    Box::new(move |m, gfx| {
                        let vert = &m[0];
                        let vb = &[MeshVertex::desc()];

                        gfx.depth_pipeline(vb, vert, None, smap, double_sided)
                    }),
                );

                let pipeline_depth_alpha = LitMeshPipeline {
                    alpha: true,
                    smap,
                    depth: true,
                    double_sided,
                };
                gfx.register_pipeline(
                    pipeline_depth_alpha,
                    &["lit_mesh.vert", "alpha_discard.frag"],
                    Box::new(move |m, gfx| {
                        let vert = &m[0];
                        let frag = &m[1];
                        let vb = &[MeshVertex::desc()];

                        gfx.depth_pipeline_bglayout(
                            vb,
                            vert,
                            Some(frag),
                            smap,
                            &[
                                &gfx.projection.layout,
                                &Texture::bindgroup_layout(&gfx.device),
                            ],
                            double_sided,
                        )
                    }),
                );
            }
        }
    }
}

impl Drawable for Mesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        rp.set_pipeline(gfx.get_pipeline(LitMeshPipeline {
            alpha: false,
            smap: false,
            depth: false,
            double_sided: self.double_sided,
        }));
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.render_params.bindgroup, &[]);
        rp.set_bind_group(2, &gfx.material(self.material).bg, &[]);
        rp.set_bind_group(3, &gfx.simplelit_bg, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..self.n_indices, 0, 0..1);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        if self.skip_depth {
            return;
        }
        rp.set_pipeline(gfx.get_pipeline(LitMeshPipeline {
            alpha: self.transparent,
            smap: shadow_map,
            depth: true,
            double_sided: self.double_sided,
        }));

        rp.set_bind_group(0, proj, &[]);
        if self.transparent {
            rp.set_bind_group(1, &gfx.material(self.material).bg, &[]);
        }
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}

pub struct LitMeshDepth;
pub struct LitMeshDepthSMap;

pub enum BgLayoutTextureType {
    Shadow,
    Float,
}

pub fn bg_layout_litmesh(device: &Device) -> BindGroupLayout {
    use BgLayoutTextureType::*;
    bg_layout_texs(device, [Float, Float, Shadow].into_iter())
}
pub fn bg_layout_texs(
    device: &Device,
    it: impl Iterator<Item = BgLayoutTextureType>,
) -> BindGroupLayout {
    let entries: Vec<BindGroupLayoutEntry> = it
        .enumerate()
        .flat_map(|(i, bgtype)| {
            vec![
                BindGroupLayoutEntry {
                    binding: (i * 2) as u32,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: if matches!(bgtype, BgLayoutTextureType::Shadow) {
                            wgpu::TextureSampleType::Depth
                        } else {
                            wgpu::TextureSampleType::Float { filterable: true }
                        },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: (i * 2 + 1) as u32,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(
                        if matches!(bgtype, BgLayoutTextureType::Shadow) {
                            wgpu::SamplerBindingType::Comparison
                        } else {
                            wgpu::SamplerBindingType::Filtering
                        },
                    ),
                    count: None,
                },
            ]
            .into_iter()
        })
        .collect::<Vec<_>>();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &entries,
        label: Some("Texture bindgroup layout"),
    })
}
