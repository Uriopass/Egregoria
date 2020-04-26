use crate::engine::{
    compile_shader, CompiledShader, Drawable, FrameContext, GfxContext, IndexType, Texture,
    UvVertex, VBDesc,
};

use lazy_static::*;
use wgpu::{TextureComponentType, VertexBufferDescriptor};

pub struct SpriteBatchBuilder {
    pub tex: Texture,
    pub instances: Vec<InstanceRaw>,
}

pub struct SpriteBatch {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    pub n_indices: u32,
    pub n_instances: u32,
    pub alpha_blend: bool,
    pub tex: Texture,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct InstanceRaw {
    model: cgmath::Matrix4<f32>,
    tint: cgmath::Vector3<f32>,
}

impl InstanceRaw {
    pub fn new(
        mut model: cgmath::Matrix4<f32>,
        tint: cgmath::Vector3<f32>,
        scale: f32,
    ) -> InstanceRaw {
        model.x.x *= scale;
        model.x.y *= scale;
        model.y.x *= scale;
        model.y.y *= scale;
        Self { model, tint }
    }
}

u8slice_impl!(InstanceRaw);

impl VBDesc for InstanceRaw {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![2 => Float4, 3 => Float4, 4 => Float4, 5 => Float4, 6 => Float3],
        }
    }
}

const UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [0.0, 0.0, 0.0],
        uv: [0.0, 1.0],
    },
    UvVertex {
        position: [1.0, 0.0, 0.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    UvVertex {
        position: [0.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[0, 1, 2, 0, 2, 3];

impl SpriteBatchBuilder {
    pub fn new(tex: Texture) -> Self {
        Self {
            tex,
            instances: vec![],
        }
    }

    pub fn build(&self, gfx: &GfxContext) -> Option<SpriteBatch> {
        let pipeline = gfx.get_pipeline::<SpriteBatch>();

        let m = self.tex.width.max(self.tex.height);

        let x = self.tex.width / (2.0 * m);
        let y = self.tex.height / (2.0 * m);

        let v = [
            UvVertex {
                position: [-x, -y, 0.0],
                ..UV_VERTICES[0]
            },
            UvVertex {
                position: [x, -y, 0.0],
                ..UV_VERTICES[1]
            },
            UvVertex {
                position: [x, y, 0.0],
                ..UV_VERTICES[2]
            },
            UvVertex {
                position: [-x, y, 0.0],
                ..UV_VERTICES[3]
            },
        ];

        if self.instances.len() == 0 {
            return None;
        }

        let vertex_buffer = gfx
            .device
            .create_buffer_with_data(bytemuck::cast_slice(&v), wgpu::BufferUsage::VERTEX);
        let index_buffer = gfx
            .device
            .create_buffer_with_data(bytemuck::cast_slice(UV_INDICES), wgpu::BufferUsage::INDEX);
        let instance_buffer = gfx.device.create_buffer_with_data(
            bytemuck::cast_slice(&self.instances),
            wgpu::BufferUsage::VERTEX,
        );

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.bindgroupslayouts[0],
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.tex.view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.tex.sampler),
                },
            ],
            label: None,
        });

        Some(SpriteBatch {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            n_indices: UV_INDICES.len() as u32,
            n_instances: self.instances.len() as u32,
            alpha_blend: false,
            tex: self.tex.clone(),
            bind_group,
        })
    }
}

lazy_static! {
    static ref VERT_SHADER: CompiledShader = compile_shader("resources/shaders/spritebatch.vert");
    static ref FRAG_SHADER: CompiledShader = compile_shader("resources/shaders/spritebatch.frag");
}

impl Drawable for SpriteBatch {
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
                vertex_buffers: &[UvVertex::desc(), InstanceRaw::desc()],
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

        let pipeline = &ctx.gfx.get_pipeline::<Self>();
        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_bind_group(1, &ctx.gfx.projection.bindgroup, &[]);
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);
        render_pass.set_vertex_buffer(1, &self.instance_buffer, 0, 0);
        render_pass.set_index_buffer(&self.index_buffer, 0, 0);
        render_pass.draw_indexed(0..self.n_indices, 0, 0..self.n_instances);
    }
}
