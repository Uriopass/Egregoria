use crate::{compile_shader, ColoredVertex, Drawable, GfxContext, IndexType, VBDesc};
use std::rc::Rc;
use wgpu::util::DeviceExt;
use wgpu::{IndexFormat, RenderPass, RenderPipeline};

#[derive(Default)]
pub struct MeshBuilder {
    pub vertices: Vec<ColoredVertex>,
    pub indices: Vec<IndexType>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn extend(&mut self, vertices: &[ColoredVertex], indices: &[IndexType]) -> &mut Self {
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|x| x + offset));
        self
    }

    #[inline(always)]
    pub fn extend_with(&mut self, f: impl Fn(&mut Vec<ColoredVertex>, &mut dyn FnMut(IndexType))) {
        let offset = self.vertices.len() as IndexType;
        let vertices = &mut self.vertices;
        let indices = &mut self.indices;
        let mut x = move |index: IndexType| {
            indices.push(index + offset);
        };
        f(vertices, &mut x);
    }

    pub fn build(self, ctx: &GfxContext) -> Option<Mesh> {
        if self.vertices.is_empty() {
            return None;
        }
        let vertex_buffer = Rc::new(ctx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsage::VERTEX,
            },
        ));
        let index_buffer = Rc::new(ctx.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsage::INDEX,
            },
        ));

        Some(Mesh {
            vertex_buffer,
            index_buffer,
            n_indices: self.indices.len() as u32,
            alpha_blend: false,
        })
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertex_buffer: Rc<wgpu::Buffer>,
    pub index_buffer: Rc<wgpu::Buffer>,
    pub n_indices: u32,
    pub alpha_blend: bool,
}

impl Drawable for Mesh {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline {
        let vert = compile_shader("assets/shaders/mesh_shader.vert", None);
        let frag = compile_shader("assets/shaders/mesh_shader.frag", None);

        gfx.basic_pipeline(
            &[&gfx.projection.layout],
            &[ColoredVertex::desc()],
            vert,
            frag,
        )
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        rp.set_pipeline(&gfx.get_pipeline::<Self>());
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}
