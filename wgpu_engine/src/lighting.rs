use crate::pbuffer::PBuffer;
use crate::{compile_shader, GfxContext, UvVertex, VBDesc};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BlendFactor, Buffer, BufferUsage, CommandEncoder, IndexFormat, VertexAttribute,
    VertexBufferLayout,
};

pub struct LightRender {
    vertex_buffer: Buffer,
    instance_buffer: PBuffer,
}

impl LightRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        // ok: init
        let vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        Self {
            vertex_buffer,
            instance_buffer: PBuffer::new(BufferUsage::VERTEX),
        }
    }
}

struct LightBlit;

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
        blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::REPLACE,
        }),
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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightInstance {
    pub pos: [f32; 2],
    pub scale: f32,
}

u8slice_impl!(LightInstance);

const ATTRS: &[VertexAttribute] = &wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32];

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
        lights: &[LightInstance],
    ) {
        self.instance_buffer
            .write(gfx, bytemuck::cast_slice(lights));

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &gfx.fbos.light.view,
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
            rpass.draw_indexed(0..6, 0, 0..lights.len() as u32);
        }
    }
}
