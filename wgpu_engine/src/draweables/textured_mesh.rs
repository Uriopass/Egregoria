#![allow(dead_code)]
use crate::{compile_shader, ColoredUvVertex, Drawable, GfxContext, IndexType, Texture, VBDesc};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{RenderPass, RenderPipeline, TextureComponentType};

#[derive(Default)]
pub struct TexturedMeshBuilder {
    vertices: Vec<ColoredUvVertex>,
    indices: Vec<IndexType>,
}

pub struct TexturedMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub n_indices: u32,
    pub alpha_blend: bool,
    pub tex: Texture,
    pub bind_group: wgpu::BindGroup,
}

impl TexturedMeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn extend(&mut self, vertices: &[ColoredUvVertex], indices: &[IndexType]) -> &mut Self {
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|x| x + offset));
        self
    }

    pub fn build(&self, gfx: &GfxContext, tex: Texture) -> Option<TexturedMesh> {
        if self.vertices.is_empty() {
            return None;
        }

        let pipeline = gfx.get_pipeline::<TexturedMesh>();

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

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(0),
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
            label: Some("Textured mesh bindgroup"),
        });

        Some(TexturedMesh {
            vertex_buffer,
            index_buffer,
            n_indices: self.indices.len() as u32,
            alpha_blend: false,
            tex,
            bind_group,
        })
    }
}

impl Drawable for TexturedMesh {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline {
        let layouts = vec![Texture::bindgroup_layout(
            &gfx.device,
            TextureComponentType::Uint,
        )];

        let vert = compile_shader("assets/shaders/textured_mesh_shader.vert", None);
        let frag = compile_shader("assets/shaders/textured_mesh_shader.frag", None);

        gfx.basic_pipeline(
            &[&layouts[0], &gfx.projection.layout],
            &[ColoredUvVertex::desc()],
            vert,
            frag,
        )
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let pipeline = &gfx.get_pipeline::<Self>();
        rp.set_pipeline(&pipeline);
        rp.set_bind_group(0, &self.bind_group, &[]);
        rp.set_bind_group(1, &gfx.projection.bindgroup, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..));
        rp.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}
