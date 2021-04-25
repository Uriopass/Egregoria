use crate::pbuffer::PBuffer;
use crate::{
    compile_shader, GfxContext, IndexType, LightParams, Texture, TextureBuilder, Uniform, UvVertex,
    VBDesc,
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    AddressMode, BlendFactor, Buffer, BufferUsage, CommandEncoder, FilterMode, IndexFormat,
    MultisampleState, SamplerDescriptor, SwapChainFrame, TextureSampleType, VertexAttribute,
    VertexBufferLayout,
};

pub struct LightRender {
    blue_noise: Texture,
    vertex_buffer: Buffer,
    instance_buffer: PBuffer,
}

impl LightRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        let mut blue_noise = TextureBuilder::from_path("assets/blue_noise_512.png")
            .with_label("blue noise")
            .with_srgb(false)
            .with_mipmaps(false)
            .build(gfx);
        blue_noise.sampler = gfx.device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        // ok: init
        let vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Self {
            vertex_buffer,
            blue_noise,
            instance_buffer: PBuffer::new(BufferUsage::VERTEX),
        }
    }
}

struct LightBlit;
struct LightMultiply;

pub fn setup(gfx: &mut GfxContext) {
    let vert_shader = compile_shader(&gfx.device, "assets/shaders/blit_light.vert", None);
    let frag_shader = compile_shader(&gfx.device, "assets/shaders/blit_light.frag", None);

    let render_pipeline_layout =
        gfx.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("light blit layout"),
                bind_group_layouts: &[&gfx.projection.layout],
                push_constant_ranges: &[],
            });

    let color_states = [wgpu::ColorTargetState {
        format: gfx.fbos.light.format,
        color_blend: wgpu::BlendState {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
        alpha_blend: wgpu::BlendState::REPLACE,
        write_mask: wgpu::ColorWrite::ALL,
    }];

    let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
        label: Some("light blit"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vert_shader.0,
            entry_point: "main",
            buffers: &[UvVertex::desc(), LightInstance::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &frag_shader.0,
            entry_point: "main",
            targets: &color_states,
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
    };

    let pipe = gfx.device.create_render_pipeline(&render_pipeline_desc);
    gfx.register_pipeline::<LightBlit>(pipe);

    let render_pipeline_layout =
        gfx.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("light multiply layout"),
                bind_group_layouts: &[
                    &Texture::bindgroup_layout_complex(
                        &gfx.device,
                        TextureSampleType::Float { filterable: true },
                        3,
                    ),
                    &Uniform::<LightParams>::bindgroup_layout(&gfx.device),
                ],
                push_constant_ranges: &[],
            });

    let vs_module = compile_shader(&gfx.device, "assets/shaders/light_multiply.vert", None).0;
    let fs_module = compile_shader(&gfx.device, "assets/shaders/light_multiply.frag", None).0;

    let color_states = [wgpu::ColorTargetState {
        format: gfx.sc_desc.format,
        color_blend: wgpu::BlendState::REPLACE,
        alpha_blend: wgpu::BlendState::REPLACE,
        write_mask: wgpu::ColorWrite::ALL,
    }];

    let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
        label: Some("light multiply"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "main",
            buffers: &[UvVertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: "main",
            targets: &color_states,
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: MultisampleState::default(),
    };
    let pipe = gfx.device.create_render_pipeline(&render_pipeline_desc);
    gfx.register_pipeline::<LightMultiply>(pipe);
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

const UV_INDICES: &[IndexType] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightInstance {
    pub pos: [f32; 2],
    pub scale: f32,
}

u8slice_impl!(LightInstance);

const ATTRS: &[VertexAttribute] = &wgpu::vertex_attr_array![2 => Float2, 3 => Float];

impl VBDesc for LightInstance {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LightInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: ATTRS,
        }
    }
}

impl LightRender {
    pub fn render_lights(
        &mut self,
        gfx: &GfxContext,
        encoder: &mut CommandEncoder,
        frame: &SwapChainFrame,
        lights: &[LightInstance],
    ) {
        self.instance_buffer
            .write(gfx, bytemuck::cast_slice(lights));

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &gfx.fbos.light.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.25,
                            g: 0.25,
                            b: 0.25,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            if let Some(ref instance_buffer) = self.instance_buffer.inner() {
                rpass.set_pipeline(&gfx.get_pipeline::<LightBlit>());
                rpass.set_bind_group(0, &gfx.projection.bindgroup, &[]);
                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                rpass.set_vertex_buffer(1, instance_buffer.slice(..));
                rpass.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
                rpass.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..lights.len() as u32);
            }
        }

        let lmultiply_tex_bg = Texture::multi_bindgroup(
            &[&gfx.fbos.light, &gfx.fbos.color.target, &self.blue_noise],
            &gfx.device,
            &gfx.get_pipeline::<LightMultiply>().get_bind_group_layout(0),
        );

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&gfx.get_pipeline::<LightMultiply>());
        rpass.set_bind_group(0, &lmultiply_tex_bg, &[]);
        rpass.set_bind_group(1, &gfx.light_params.bindgroup, &[]);
        rpass.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
        rpass.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
        rpass.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..1);
    }
}
