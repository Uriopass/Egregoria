use crate::engine::{
    compile_shader, CompiledShader, Draweable, FrameContext, GfxContext, IndexType, Uniform, Vertex,
};
use lazy_static::*;
use wgpu::ShaderStage;

pub struct RainbowMeshBuilder {
    vertices: Vec<Vertex>,
    indices: Vec<IndexType>,
}

impl RainbowMeshBuilder {
    pub fn new() -> Self {
        Self {
            vertices: vec![],
            indices: vec![],
        }
    }

    pub fn extend(&mut self, vertices: &[Vertex], indices: &[IndexType]) -> &mut Self {
        let offset = self.vertices.len() as IndexType;
        self.vertices.extend_from_slice(vertices);
        self.indices.extend(indices.into_iter().map(|x| x + offset));
        self
    }

    pub fn build(self, gfx: &GfxContext) -> RainbowMesh {
        let pipeline = gfx.get_pipeline::<RainbowMesh>();

        let vertex_buffer = gfx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.vertices),
            wgpu::BufferUsage::VERTEX,
        );
        let index_buffer = gfx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.indices),
            wgpu::BufferUsage::INDEX,
        );

        let time = TimeUniform { time: 0.0 };

        let time_uniform_buffer = gfx.device.create_buffer_with_data(
            bytemuck::cast_slice(&[time]),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        let time_uniform_bindgroup = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bindgroupslayouts[0],
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &time_uniform_buffer,
                    // FYI: you can share a single buffer between bindings.
                    range: 0..std::mem::size_of_val(&time) as wgpu::BufferAddress,
                },
            }],
            label: None,
        });

        RainbowMesh {
            vertex_buffer,
            index_buffer,
            n_indices: self.indices.len() as u32,
            alpha_blend: false,
            time: Uniform {
                buffer: time_uniform_buffer,
                bindgroup: time_uniform_bindgroup,
                value: TimeUniform { time: 0.0 },
            },
        }
    }
}

pub struct RainbowMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub n_indices: u32,
    pub alpha_blend: bool,
    pub time: Uniform<TimeUniform>,
}

#[derive(Clone, Copy)]
pub struct TimeUniform {
    pub time: f32,
}

unsafe impl bytemuck::Pod for TimeUniform {}
unsafe impl bytemuck::Zeroable for TimeUniform {}

lazy_static! {
    static ref VERT_SHADER: CompiledShader =
        compile_shader("resources/shaders/rainbow_mesh_shader.vert");
    static ref FRAG_SHADER: CompiledShader =
        compile_shader("resources/shaders/rainbow_mesh_shader.frag");
}

impl Draweable for RainbowMesh {
    fn create_pipeline(gfx: &GfxContext) -> super::PreparedPipeline {
        let layouts = vec![Uniform::<TimeUniform>::bindgroup_layout(
            &gfx.device,
            0,
            ShaderStage::FRAGMENT,
        )];

        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&layouts[0]],
                });

        let vs_module = gfx.device.create_shader_module(&VERT_SHADER.0);
        let fs_module = gfx.device.create_shader_module(&FRAG_SHADER.0);

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
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &color_states,
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[Vertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };
        super::PreparedPipeline {
            pipeline: gfx.device.create_render_pipeline(&render_pipeline_desc),
            bindgroupslayouts: layouts,
        }
    }

    fn draw(&self, ctx: &mut FrameContext) {
        self.time.upload_to_gpu(&ctx.gfx.device, &mut ctx.encoder);

        let mut render_pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &ctx.frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &ctx.gfx.depth_texture.view,
                depth_load_op: wgpu::LoadOp::Load,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Load,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });

        render_pass.set_pipeline(&ctx.gfx.get_pipeline::<Self>().pipeline);
        render_pass.set_bind_group(0, &self.time.bindgroup, &[]);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);
        render_pass.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}
