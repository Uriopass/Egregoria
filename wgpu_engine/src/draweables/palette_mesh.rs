#![allow(dead_code)]
use crate::pbuffer::PBuffer;
use crate::{
    compile_shader, Drawable, GfxContext, IndexType, LightParams, NorUvVertex, Texture, Uniform,
    VBDesc,
};
use geom::{LinearColor, Vec3};
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BufferUsage, IndexFormat, RenderPass, RenderPipeline, VertexAttribute, VertexBufferLayout,
};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MeshInstance {
    pub pos: Vec3,
    pub dir: Vec3,
    pub tint: LinearColor,
}

u8slice_impl!(MeshInstance);

const ATTRS: &[VertexAttribute] = &wgpu::vertex_attr_array![3 => Float3, 4 => Float3, 5 => Float4];

impl VBDesc for MeshInstance {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

#[derive(Default)]
pub struct PaletteMeshBuilder {
    pub vertices: Vec<NorUvVertex>,
    pub indices: Vec<IndexType>,
}

pub struct PaletteMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub n_indices: u32,
    pub palette: Arc<Texture>,
    pub texbg: wgpu::BindGroup,
}

pub struct InstancedPaletteMeshBuilder {
    mesh: Arc<PaletteMesh>,
    ibuffer: PBuffer,
    pub instances: Vec<MeshInstance>,
}

impl InstancedPaletteMeshBuilder {
    pub fn new(mesh: Arc<PaletteMesh>) -> Self {
        InstancedPaletteMeshBuilder {
            mesh,
            instances: vec![],
            ibuffer: PBuffer::new(BufferUsage::VERTEX),
        }
    }

    pub fn build(&mut self, gfx: &GfxContext) -> Option<InstancedPaletteMesh> {
        if self.instances.is_empty() {
            return None;
        }

        self.ibuffer
            .write(gfx, bytemuck::cast_slice(&self.instances));

        Some(InstancedPaletteMesh {
            mesh: self.mesh.clone(),
            instance_buffer: self.ibuffer.inner()?,
            n_instances: self.instances.len() as u32,
        })
    }
}

#[derive(Clone)]
pub struct InstancedPaletteMesh {
    mesh: Arc<PaletteMesh>,
    instance_buffer: Arc<wgpu::Buffer>,
    n_instances: u32,
}

impl PaletteMeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn extend(&mut self, vertices: &[NorUvVertex], indices: &[IndexType]) -> &mut Self {
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|x| x + offset));
        self
    }

    pub fn extend_raw(&mut self, vertices: &[NorUvVertex], indices: &[IndexType]) {
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices);
    }

    pub fn build(&self, gfx: &GfxContext) -> Option<PaletteMesh> {
        if self.vertices.is_empty() {
            return None;
        }

        let pipeline = gfx.get_pipeline::<InstancedPaletteMesh>();

        let vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsage::INDEX,
        });

        let tex = gfx.palette().clone();

        let texbg = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(2),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                },
            ],
            label: Some("Palette mesh bindgroup"),
        });

        Some(PaletteMesh {
            vertex_buffer,
            index_buffer,
            n_indices: self.indices.len() as u32,
            palette: tex,
            texbg,
        })
    }
}

impl PaletteMesh {
    pub fn builder() -> PaletteMeshBuilder {
        PaletteMeshBuilder::new()
    }
}

impl Drawable for InstancedPaletteMesh {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline {
        let vert = compile_shader(&gfx.device, "assets/shaders/palette.vert", None);
        let frag = compile_shader(&gfx.device, "assets/shaders/simple_lit.frag", None);

        gfx.basic_pipeline(
            &[
                &gfx.projection.layout,
                &Uniform::<LightParams>::bindgroup_layout(&gfx.device),
                &Texture::bindgroup_layout(&gfx.device),
            ],
            &[NorUvVertex::desc(), MeshInstance::desc()],
            vert,
            frag,
        )
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline);
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.light_params.bindgroup, &[]);
        rp.set_bind_group(2, &self.mesh.texbg, &[]);
        rp.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        rp.set_vertex_buffer(1, self.instance_buffer.slice(..));
        rp.set_index_buffer(self.mesh.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..self.mesh.n_indices, 0, 0..self.n_instances);
    }
}
