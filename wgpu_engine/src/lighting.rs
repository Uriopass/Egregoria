use crate::{
    compile_shader, Drawable, GfxContext, IndexType, PreparedPipeline, Texture, Uniform, UvVertex,
    VBDesc,
};
use geom::Vec3;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BlendFactor, CommandEncoder, RenderPass, SwapChainFrame, TextureComponentType,
    VertexBufferDescriptor,
};

struct LightBlit;
struct LightMultiply;

impl Drawable for LightBlit {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline
    where
        Self: Sized,
    {
        let vert_shader = compile_shader("assets/shaders/blit_light.vert", None);
        let frag_shader = compile_shader("assets/shaders/blit_light.frag", None);

        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("light pipeline"),
                    bind_group_layouts: &[&gfx.projection.layout],
                    push_constant_ranges: &[],
                });

        let vs_module = gfx.device.create_shader_module(vert_shader.0);
        let fs_module = gfx.device.create_shader_module(frag_shader.0);

        let color_states = [wgpu::ColorStateDescriptor {
            format: gfx.light_texture.format,
            color_blend: wgpu::BlendDescriptor {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
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
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[UvVertex::desc(), LightInstance::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };

        PreparedPipeline(gfx.device.create_render_pipeline(&render_pipeline_desc))
    }

    fn draw<'a>(&'a self, _gfx: &'a GfxContext, _rp: &mut RenderPass<'a>) {
        unimplemented!()
    }
}

impl Drawable for LightMultiply {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline
    where
        Self: Sized,
    {
        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("basic pipeline"),
                    bind_group_layouts: &[
                        &Texture::bindgroup_layout(&gfx.device, TextureComponentType::Float),
                        &Texture::bindgroup_layout(&gfx.device, TextureComponentType::Float),
                        &Uniform::<Vec3>::bindgroup_layout(
                            &gfx.device,
                            wgpu::ShaderStage::FRAGMENT,
                        ),
                    ],
                    push_constant_ranges: &[],
                });

        let vs_module = gfx
            .device
            .create_shader_module(compile_shader("assets/shaders/light_multiply.vert", None).0);
        let fs_module = gfx
            .device
            .create_shader_module(compile_shader("assets/shaders/light_multiply.frag", None).0);

        let color_states = [wgpu::ColorStateDescriptor {
            format: gfx.sc_desc.format,
            color_blend: wgpu::BlendDescriptor::REPLACE,
            alpha_blend: wgpu::BlendDescriptor::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
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
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[UvVertex::desc()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };
        PreparedPipeline(gfx.device.create_render_pipeline(&render_pipeline_desc))
    }

    fn draw<'a>(&'a self, _gfx: &'a GfxContext, _rp: &mut RenderPass<'a>) {
        unimplemented!()
    }
}

const UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [-1.0, -1.0, 0.0],
        uv: [-1.0, 1.0],
    },
    UvVertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, -1.0],
    },
    UvVertex {
        position: [-1.0, 1.0, 0.0],
        uv: [-1.0, -1.0],
    },
];

const SCREEN_UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [-1.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    UvVertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    UvVertex {
        position: [-1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[0, 1, 2, 0, 2, 3];

pub fn prepare_lighting(gfx: &mut GfxContext) {
    gfx.register_pipeline::<LightBlit>();
    gfx.register_pipeline::<LightMultiply>();
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LightInstance {
    pub pos: [f32; 2],
    pub scale: f32,
}

u8slice_impl!(LightInstance);

impl VBDesc for LightInstance {
    fn desc<'a>() -> VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<LightInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: Box::leak(Box::new(wgpu::vertex_attr_array![2 => Float2, 3 => Float])),
        }
    }
}

pub fn render_lights(
    gfx: &GfxContext,
    encoder: &mut CommandEncoder,
    frame: &SwapChainFrame,
    lights: &[LightInstance],
    ambiant: Vec3,
) {
    let vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(UV_VERTICES),
        usage: wgpu::BufferUsage::VERTEX,
    });

    let index_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(UV_INDICES),
        usage: wgpu::BufferUsage::INDEX,
    });

    let instance_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(lights),
        usage: wgpu::BufferUsage::VERTEX,
    });

    {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &gfx.light_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&gfx.get_pipeline::<LightBlit>().0);
        rpass.set_bind_group(0, &gfx.projection.bindgroup, &[]);
        rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
        rpass.set_vertex_buffer(1, instance_buffer.slice(..));
        rpass.set_index_buffer(index_buffer.slice(..));
        rpass.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..lights.len() as u32);
    }

    let vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(SCREEN_UV_VERTICES),
        usage: wgpu::BufferUsage::VERTEX,
    });

    let light_tex_bind_group = gfx.light_texture.bindgroup(
        &gfx.device,
        &gfx.get_pipeline::<LightMultiply>()
            .0
            .get_bind_group_layout(0),
    );

    let color_tex_bind_group = gfx.color_texture.target.bindgroup(
        &gfx.device,
        &gfx.get_pipeline::<LightMultiply>()
            .0
            .get_bind_group_layout(1),
    );

    let ambiant_uni = Uniform::new(
        [ambiant.x, ambiant.y, ambiant.z, gfx.time_uni.value],
        &gfx.device,
        wgpu::ShaderStage::FRAGMENT,
    );

    ambiant_uni.upload_to_gpu(&gfx.queue);

    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
            attachment: &frame.output.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        }],
        depth_stencil_attachment: None,
    });

    rpass.set_pipeline(&gfx.get_pipeline::<LightMultiply>().0);
    rpass.set_bind_group(0, &light_tex_bind_group, &[]);
    rpass.set_bind_group(1, &color_tex_bind_group, &[]);
    rpass.set_bind_group(2, &ambiant_uni.bindgroup, &[]);
    rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
    rpass.set_index_buffer(index_buffer.slice(..));
    rpass.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..1);
}
