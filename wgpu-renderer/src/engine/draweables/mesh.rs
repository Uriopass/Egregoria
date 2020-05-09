use crate::engine::{
    compile_shader, CompiledShader, Drawable, GfxContext, HasPipeline, IndexType, PreparedPipeline,
    VBDesc, Vertex,
};
use lazy_static::*;
use std::rc::Rc;
use wgpu::RenderPass;

pub struct MeshBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<IndexType>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn extend(&mut self, vertices: &[Vertex], indices: &[IndexType]) -> &mut Self {
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.iter().map(|x| x + offset));
        self
    }

    pub fn extend_with(&mut self, f: impl Fn(&mut Vec<Vertex>, &mut dyn FnMut(IndexType)) -> ()) {
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

lazy_static! {
    static ref VERT_SHADER: CompiledShader = compile_shader("resources/shaders/mesh_shader.vert");
    static ref FRAG_SHADER: CompiledShader = compile_shader("resources/shaders/mesh_shader.frag");
}

impl HasPipeline for Mesh {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline {
        let vs_module = gfx.device.create_shader_module(&VERT_SHADER.0);
        let fs_module = gfx.device.create_shader_module(&FRAG_SHADER.0);

        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&gfx.projection_layout],
                });
        let color_states = [wgpu::ColorStateDescriptor {
            format: gfx.sc_desc.format,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];
        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &color_states,
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: gfx.samples,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };
        PreparedPipeline {
            pipeline: gfx.device.create_render_pipeline(&render_pipeline_desc),
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
