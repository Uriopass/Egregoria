use crate::engine::{
    compile_shader, ColoredUvVertex, CompiledShader, Drawable, FrameContext, GfxContext, IndexType,
    Texture, VBDesc,
};
use lazy_static::*;
use wgpu::TextureComponentType;

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

        let vertex_buffer = gfx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.vertices),
            wgpu::BufferUsage::VERTEX,
        );
        let index_buffer = gfx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.indices),
            wgpu::BufferUsage::INDEX,
        );

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bindgroupslayouts[0],
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                },
            ],
            label: None,
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

lazy_static! {
    static ref VERT_SHADER: CompiledShader =
        compile_shader("resources/shaders/textured_mesh_shader.vert");
    static ref FRAG_SHADER: CompiledShader =
        compile_shader("resources/shaders/textured_mesh_shader.frag");
}

impl Drawable for TexturedMesh {
    fn create_pipeline(gfx: &GfxContext) -> super::PreparedPipeline {
        let layouts = vec![gfx
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                            component_type: TextureComponentType::Uint,
                        },
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                    },
                ],
                label: None,
            })];

        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&layouts[0], &gfx.projection_layout],
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
                vertex_buffers: &[ColoredUvVertex::desc()],
            },
            sample_count: gfx.samples,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };
        super::PreparedPipeline {
            pipeline: gfx.device.create_render_pipeline(&render_pipeline_desc),
            bindgroupslayouts: layouts,
        }
    }

    fn draw(&self, ctx: &mut FrameContext) {
        let mut render_pass = ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &ctx.gfx.multi_frame,
                resolve_target: Some(&ctx.frame.view),
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

        let pipeline = &ctx.gfx.get_pipeline::<Self>();
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &ctx.gfx.projection.bindgroup, &[]);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);
        render_pass.draw_indexed(0..self.n_indices, 0, 0..1);
    }
}
