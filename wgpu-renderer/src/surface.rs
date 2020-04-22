use std::ops::Range;

use wgpu::*;
use wgpu::{RenderPassDescriptor as WgpuRenderPassDescriptor, Surface as WgpuSurface};
use winit::window::Window;

use crate::depth::create_depth_texture;
use crate::{shader::ShaderDescriptor, vertex::Vertex};

pub struct PipelineDescriptor {
    pub shader_desc: ShaderDescriptor,
    pub vertex_buffer_number: usize,
    pub alpha_blending: bool,
    pub compare_depth_function: CompareFunction,
    pub primitive_topo: PrimitiveTopology,
}

// If you use a vertex buffer, set the `vertices` fields to None, else set `buffer_usage_desc` to None
// TODO: Replace this logic with an enum
pub struct RenderPassDescriptor<'a> {
    pub clear_color: Color,
    pub buffer_usage_desc: Option<BufferUsageDescriptor<'a>>,
    pub vertices: Option<Range<u32>>,
}

pub struct BufferUsageDescriptor<'a> {
    pub vertex_buffer: &'a Buffer,
    pub index_buffer: &'a Buffer,
    pub indices: Range<u32>,
    pub base_vertex: i32,
}

#[allow(dead_code)]
pub struct Surface {
    surface: WgpuSurface,
    size: (u32, u32),
    window: Window,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    swapchain: SwapChain,
    depth_texture: Texture,
    depth_view: TextureView,
    sc_desc: SwapChainDescriptor,
    pipeline: Option<RenderPipeline>,
}

impl Surface {
    pub async fn new(window: Window) -> Self {
        let (win_width, win_height) = (window.inner_size().width, window.inner_size().height);
        let surface = WgpuSurface::create(&window);
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        )
        .await
        .expect("Failed to find a suitable adapter");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: Default::default(),
            })
            .await;
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: win_width,
            height: win_height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let swapchain = device.create_swap_chain(&surface, &sc_desc);
        let (depth_texture, depth_view) = create_depth_texture(&device, &sc_desc);
        Self {
            size: (win_width, win_height),
            swapchain,
            device,
            queue,
            sc_desc,
            adapter,
            depth_texture,
            depth_view,
            surface,
            pipeline: None,
            window,
        }
    }
    pub fn create_pipeline(&mut self, pipe_descriptor: PipelineDescriptor) {
        let (vs_module, fs_module) = pipe_descriptor.shader_desc.into_compiled_shaders(&self);
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[],
                });
        let color_states = if pipe_descriptor.alpha_blending {
            [wgpu::ColorStateDescriptor {
                format: self.sc_desc.format,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }]
        } else {
            [wgpu::ColorStateDescriptor {
                format: self.sc_desc.format,
                color_blend: BlendDescriptor::REPLACE,
                alpha_blend: BlendDescriptor::REPLACE,
                write_mask: ColorWrite::ALL,
            }]
        };
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
            primitive_topology: pipe_descriptor.primitive_topo,
            color_states: &color_states,
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: pipe_descriptor.compare_depth_function,
                stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
                stencil_read_mask: 0,
                stencil_write_mask: 0,
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &vec![Vertex::desc(); pipe_descriptor.vertex_buffer_number],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };
        self.pipeline = Some(self.device.create_render_pipeline(&render_pipeline_desc));
    }
    pub fn redraw(&mut self, render_pass_desc: RenderPassDescriptor) {
        let frame = self
            .swapchain
            .get_next_texture()
            .expect("Timeout getting texture");

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&WgpuRenderPassDescriptor {
            color_attachments: &[RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: render_pass_desc.clear_color,
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_view,
                depth_load_op: wgpu::LoadOp::Clear,
                depth_store_op: wgpu::StoreOp::Store,
                clear_depth: 1.0,
                stencil_load_op: wgpu::LoadOp::Clear,
                stencil_store_op: wgpu::StoreOp::Store,
                clear_stencil: 0,
            }),
        });
        if let Some(pipeline) = &self.pipeline {
            render_pass.set_pipeline(pipeline);
        }
        if render_pass_desc.buffer_usage_desc.is_some() {
            let BufferUsageDescriptor {
                vertex_buffer,
                index_buffer,
                base_vertex,
                indices,
            } = render_pass_desc.buffer_usage_desc.unwrap();
            render_pass.set_vertex_buffer(0, vertex_buffer, 0, 0);
            render_pass.set_index_buffer(index_buffer, 0, 0);
            render_pass.draw_indexed(indices, base_vertex, 0..1);
        } else if render_pass_desc.vertices.is_some() {
            render_pass.draw(render_pass_desc.vertices.unwrap(), 0..1);
        } else {
            panic!("Neither vertices or buffer_usage_desc had a value");
        }
        std::mem::drop(render_pass);

        self.queue.submit(&[encoder.finish()]);
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = (new_size.width, new_size.height);
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.swapchain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        /*let (depth_texture, depth_view) = create_depth_texture(&self.device, &self.sc_desc);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;*/
    }
    pub fn create_buffer(&mut self, data: &[u8], usage: BufferUsage) -> Buffer {
        self.device.create_buffer_with_data(data, usage)
    }
    pub fn create_shader_module(&self, data: &std::fs::File) -> ShaderModule {
        let data = wgpu::read_spirv(data).unwrap();
        self.device.create_shader_module(&data)
    }
    pub fn request_redraw(&self) {
        self.window.request_redraw()
    }
}
