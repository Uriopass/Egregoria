use crate::engine::{
    compile_shader, ColoredVertex, Drawable, GfxContext, HasPipeline, IndexType, PreparedPipeline,
    VBDesc,
};
use std::rc::Rc;
use wgpu::RenderPass;

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

    pub fn extend_with(
        &mut self,
        f: impl Fn(&mut Vec<ColoredVertex>, &mut dyn FnMut(IndexType)) -> (),
    ) {
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
        let vertex_buffer = Rc::new(ctx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.vertices),
            wgpu::BufferUsage::VERTEX,
        ));
        let index_buffer = Rc::new(ctx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.indices),
            wgpu::BufferUsage::INDEX,
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

impl HasPipeline for Mesh {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline {
        let vert = compile_shader("resources/shaders/mesh_shader.vert", None);
        let frag = compile_shader("resources/shaders/mesh_shader.frag", None);

        let pipeline = gfx.basic_pipeline(
            &[&gfx.projection_layout],
            &[ColoredVertex::desc()],
            &vert,
            &frag,
        );

        PreparedPipeline {
            pipeline,
            bindgroupslayouts: vec![],
        }
    }
}

impl Drawable for Mesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        rp.set_pipeline(&gfx.get_pipeline::<Self>().pipeline);
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        rp.set_index_buffer(&self.index_buffer, 0, 0);
        rp.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}
