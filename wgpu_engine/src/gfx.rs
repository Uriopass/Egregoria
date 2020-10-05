use crate::lighting::prepare_lighting;
use crate::ShaderType;
use crate::{
    CompiledShader, Drawable, Mesh, PreparedPipeline, SpriteBatch, Texture, TexturedMesh, Uniform,
};
use raw_window_handle::HasRawWindowHandle;
use std::any::TypeId;
use std::collections::HashMap;
use wgpu::{
    Adapter, BindGroupLayout, CommandBuffer, CommandEncoder, CommandEncoderDescriptor, Device,
    Queue, RenderPipeline, StencilStateDescriptor, Surface, SwapChain, SwapChainDescriptor,
    SwapChainFrame, TextureView, VertexBufferDescriptor,
};

pub struct GfxContext {
    pub surface: Surface,
    pub size: (u32, u32),
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub swapchain: SwapChain,
    pub depth_texture: Texture,
    pub light_texture: Texture,
    pub sc_desc: SwapChainDescriptor,
    pub pipelines: HashMap<TypeId, PreparedPipeline>,
    pub queue_buffer: Vec<CommandBuffer>,
    pub projection: Uniform<mint::ColumnMatrix4<f32>>,
    pub inv_projection: Uniform<mint::ColumnMatrix4<f32>>,
    pub time_uni: Uniform<f32>,
    pub samples: u32,
    pub multi_frame: wgpu::TextureView,
}

pub struct GuiRenderContext<'a, 'b> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub rpass: Option<wgpu::RenderPass<'b>>,
}

pub struct FrameContext<'a> {
    pub gfx: &'a GfxContext,
    pub objs: &'a mut Vec<Box<dyn Drawable>>,
}

impl<'a> FrameContext<'a> {
    pub fn draw(&mut self, v: impl Drawable + 'static) {
        self.objs.push(Box::new(v))
    }
}

impl GfxContext {
    pub async fn new<W: HasRawWindowHandle>(window: &W, win_width: u32, win_height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find a suitable adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    shader_validation: true,
                },
                None,
            )
            .await
            .unwrap();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: win_width,
            height: win_height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let samples = 4;
        let (swapchain, depth_texture, light_texture, multi_frame) =
            Self::create_textures(&device, &surface, &sc_desc, samples);

        let projection = Uniform::new(mint::ColumnMatrix4::from([0.0; 16]), &device);

        let inv_projection = Uniform::new(mint::ColumnMatrix4::from([0.0; 16]), &device);

        let time_uni = Uniform::new(0.0, &device);

        let mut me = Self {
            size: (win_width, win_height),
            swapchain,
            device,
            queue,
            sc_desc,
            adapter,
            depth_texture,
            light_texture,
            surface,
            pipelines: HashMap::new(),
            queue_buffer: vec![],
            projection,
            inv_projection,
            time_uni,
            samples,
            multi_frame,
        };

        me.register_pipeline::<Mesh>();
        me.register_pipeline::<TexturedMesh>();
        me.register_pipeline::<SpriteBatch>();

        prepare_lighting(&mut me);

        me
    }

    pub fn set_time(&mut self, time: f32) {
        self.time_uni.value = time;
    }

    pub fn set_proj(&mut self, proj: mint::ColumnMatrix4<f32>) {
        self.projection.value = proj;
    }

    pub fn set_inv_proj(&mut self, proj: mint::ColumnMatrix4<f32>) {
        self.inv_projection.value = proj;
    }

    fn create_multisampled_framebuffer(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        sample_count: u32,
    ) -> wgpu::TextureView {
        let multisampled_texture_extent = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
            size: multisampled_texture_extent,
            mip_level_count: 1,
            sample_count,
            dimension: wgpu::TextureDimension::D2,
            format: sc_desc.format,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            label: Some("multisampled frame descriptor"),
        };

        device
            .create_texture(multisampled_frame_descriptor)
            .create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn start_frame(&mut self) -> CommandEncoder {
        let encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        self.projection.upload_to_gpu(&self.queue);
        self.inv_projection.upload_to_gpu(&self.queue);
        self.time_uni.upload_to_gpu(&self.queue);

        encoder
    }

    pub fn render_objs(
        &mut self,
        encoder: &mut CommandEncoder,
        frame: &SwapChainFrame,
        mut prepare: impl for<'a> FnMut(&'a mut FrameContext),
    ) {
        let mut objs = vec![];

        let mut fc = FrameContext {
            objs: &mut objs,
            gfx: &self,
        };

        prepare(&mut fc);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &self.multi_frame,
                resolve_target: Some(&frame.output.view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
        });

        for obj in fc.objs {
            obj.draw(&self, &mut render_pass);
        }
    }

    pub fn render_gui(
        &mut self,
        encoder: &mut CommandEncoder,
        frame: &SwapChainFrame,
        mut render_gui: impl FnMut(GuiRenderContext),
    ) {
        let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        render_gui(GuiRenderContext {
            device: &self.device,
            queue: &self.queue,
            rpass: Some(rpass),
        });
    }

    pub fn finish_frame(&mut self, encoder: CommandEncoder) {
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn create_textures(
        device: &Device,
        surface: &Surface,
        desc: &SwapChainDescriptor,
        samples: u32,
    ) -> (SwapChain, Texture, Texture, TextureView) {
        (
            device.create_swap_chain(surface, desc),
            Texture::create_depth_texture(device, desc, samples),
            Texture::create_light_texture(device, desc),
            Self::create_multisampled_framebuffer(desc, device, samples),
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        self.sc_desc.width = self.size.0;
        self.sc_desc.height = self.size.1;

        let (swapchain, depth, light, multi) =
            Self::create_textures(&self.device, &self.surface, &self.sc_desc, self.samples);

        self.swapchain = swapchain;
        self.depth_texture = depth;
        self.light_texture = light;
        self.multi_frame = multi;
    }

    pub fn basic_pipeline(
        &self,
        layouts: &[&BindGroupLayout],
        vertex_buffers: &[VertexBufferDescriptor],
        vert_shader: CompiledShader,
        frag_shader: CompiledShader,
    ) -> RenderPipeline {
        assert!(matches!(vert_shader.1, ShaderType::Vertex));
        assert!(matches!(frag_shader.1, ShaderType::Fragment));

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("basic pipeline"),
                    bind_group_layouts: layouts,
                    push_constant_ranges: &[],
                });

        let vs_module = self.device.create_shader_module(vert_shader.0);
        let fs_module = self.device.create_shader_module(frag_shader.0);

        let color_states = [wgpu::ColorStateDescriptor {
            format: self.sc_desc.format,
            color_blend: wgpu::BlendDescriptor {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
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
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: StencilStateDescriptor {
                    front: wgpu::StencilStateFaceDescriptor::IGNORE,
                    back: wgpu::StencilStateFaceDescriptor::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers,
            },
            sample_count: self.samples,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        };
        self.device.create_render_pipeline(&render_pipeline_desc)
    }

    pub fn get_pipeline<T: 'static + Drawable>(&self) -> &PreparedPipeline {
        &self
            .pipelines
            .get(&std::any::TypeId::of::<T>())
            .expect("Pipeline was not registered in context")
    }

    pub fn register_pipeline<T: 'static + Drawable>(&mut self) {
        self.pipelines
            .insert(std::any::TypeId::of::<T>(), T::create_pipeline(self));
    }
}
