use crate::pbuffer::PBuffer;
use crate::{
    compile_shader, ColNorVertex, Drawable, GfxContext, IndexType, LightParams, Uniform, VBDesc,
};
use std::sync::Arc;
use wgpu::{BufferUsage, IndexFormat, RenderPass, RenderPipeline};

pub struct LitMeshBuilder {
    pub vertices: Vec<ColNorVertex>,
    pub indices: Vec<IndexType>,
    pub vbuffer: PBuffer,
    pub ibuffer: PBuffer,
}

impl Default for LitMeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LitMeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
            vbuffer: PBuffer::new(BufferUsage::VERTEX),
            ibuffer: PBuffer::new(BufferUsage::INDEX),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn extend(&mut self, vertices: &[ColNorVertex], indices: &[IndexType]) -> &mut Self {
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|x| x + offset));
        self
    }

    #[inline(always)]
    pub fn extend_with(
        &mut self,
        f: impl FnOnce(&mut Vec<ColNorVertex>, &mut dyn FnMut(IndexType)),
    ) {
        let offset = self.vertices.len() as IndexType;
        let vertices = &mut self.vertices;
        let indices = &mut self.indices;
        let mut x = move |index: IndexType| {
            indices.push(index + offset);
        };
        f(vertices, &mut x);
    }

    pub fn build(&mut self, ctx: &GfxContext) -> Option<LitMesh> {
        if self.vertices.is_empty() {
            return None;
        }

        self.vbuffer
            .write(ctx, bytemuck::cast_slice(&self.vertices));
        self.ibuffer.write(ctx, bytemuck::cast_slice(&self.indices));

        Some(LitMesh {
            vertex_buffer: self.vbuffer.inner()?,
            index_buffer: self.ibuffer.inner()?,
            n_indices: self.indices.len() as u32,
        })
    }
}

#[derive(Clone)]
pub struct LitMesh {
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub n_indices: u32,
}

impl Drawable for LitMesh {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline {
        let vert = compile_shader(&gfx.device, "assets/shaders/lit_mesh.vert", None);
        let frag = compile_shader(&gfx.device, "assets/shaders/simple_lit.frag", None);

        gfx.basic_pipeline(
            &[
                &gfx.projection.layout,
                &Uniform::<LightParams>::bindgroup_layout(&gfx.device),
            ],
            &[ColNorVertex::desc()],
            vert,
            frag,
        )
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        rp.set_pipeline(&gfx.get_pipeline::<Self>());
        rp.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rp.set_bind_group(1, &gfx.light_params.bindgroup, &[]);
        rp.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rp.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rp.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}
