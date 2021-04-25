use crate::{
    BlitLinear, CompiledShader, Drawable, IndexType, InstancedMesh, Mesh, SpriteBatch, Texture,
    Uniform, UvVertex,
};
use crate::{MultisampledTexture, ShaderType};
use common::FastMap;
use geom::{LinearColor, Vec3};
use mint::ColumnMatrix4;
use raw_window_handle::HasRawWindowHandle;
use std::any::TypeId;
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, BindGroupLayout, CommandEncoder, CommandEncoderDescriptor, CullMode, Device,
    FrontFace, IndexFormat, MultisampleState, PrimitiveState, Queue, RenderPipeline, Surface,
    SwapChain, SwapChainDescriptor, SwapChainFrame, VertexBufferLayout,
};

pub struct GfxContext {
    pub(crate) surface: Surface,
    pub size: (u32, u32),
    #[allow(dead_code)] // keep adapter alive
    pub(crate) adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub swapchain: SwapChain,
    pub(crate) depth_texture: Texture,
    pub(crate) light_texture: Texture,
    pub(crate) color_texture: MultisampledTexture,
    pub(crate) ui_texture: Texture,
    pub(crate) sc_desc: SwapChainDescriptor,
    pub update_sc: bool,
    pub(crate) pipelines: FastMap<TypeId, RenderPipeline>,
    pub(crate) projection: Uniform<mint::ColumnMatrix4<f32>>,
    pub inv_projection: Uniform<mint::ColumnMatrix4<f32>>,
    pub time_uni: Uniform<f32>,
    pub light_params: Uniform<LightParams>,
    pub(crate) textures: FastMap<PathBuf, Arc<Texture>>,
    pub(crate) samples: u32,
    pub(crate) screen_uv_vertices: wgpu::Buffer,
    pub(crate) rect_indices: wgpu::Buffer,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LightParams {
    pub inv_proj: ColumnMatrix4<f32>,
    pub ambiant: LinearColor,
    pub cam_pos: Vec3,
    pub _pad: f32,
    pub sun: Vec3,
    pub _pad2: f32,
    pub time: f32,
}

impl Default for LightParams {
    fn default() -> Self {
        Self {
            inv_proj: ColumnMatrix4::from([0.0; 16]),
            ambiant: Default::default(),
            cam_pos: Default::default(),
            _pad: 0.0,
            sun: Default::default(),
            _pad2: 0.0,
            time: 0.0,
        }
    }
}

u8slice_impl!(LightParams);

pub struct GuiRenderContext<'a, 'b> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub rpass: Option<wgpu::RenderPass<'b>>,
}

pub struct FrameContext<'a> {
    pub gfx: &'a mut GfxContext,
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
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .expect(
                "failed to find a suitable adapter, have you installed necessary vulkan libraries?",
            );
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("could not find device, have you installed necessary vulkan libraries?");
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: win_width,
            height: win_height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let samples = 4;
        let (swapchain, depth_texture, light_texture, color_texture, ui_texture) =
            Self::create_textures(&device, &surface, &sc_desc, samples);

        let projection = Uniform::new(mint::ColumnMatrix4::from([0.0; 16]), &device);

        let inv_projection = Uniform::new(mint::ColumnMatrix4::from([0.0; 16]), &device);

        let time_uni = Uniform::new(0.0, &device);
        let screen_uv_vertices = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(SCREEN_UV_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let rect_indices = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let mut me = Self {
            size: (win_width, win_height),
            swapchain,
            queue,
            sc_desc,
            update_sc: false,
            adapter,
            depth_texture,
            color_texture,
            light_texture,
            ui_texture,
            surface,
            pipelines: FastMap::default(),
            projection,
            inv_projection,
            time_uni,
            light_params: Uniform::new(Default::default(), &device),
            textures: FastMap::default(),
            samples,
            screen_uv_vertices,
            rect_indices,
            device,
        };

        Mesh::setup(&mut me);
        InstancedMesh::setup(&mut me);
        SpriteBatch::setup(&mut me);
        crate::lighting::setup(&mut me);
        BlitLinear::setup(&mut me);

        let mut p = Texture::from_path(&me, "assets/palette.png", Some("palette"));
        p.sampler = Texture::nearest_sampler(&me.device);
        me.set_texture("assets/palette.png", p);

        me
    }

    pub fn set_texture(&mut self, path: impl Into<PathBuf>, tex: Texture) {
        let p = path.into();
        self.textures.insert(p, Arc::new(tex));
    }

    pub fn texture(
        &mut self,
        path: impl Into<PathBuf>,
        label: Option<&'static str>,
    ) -> Arc<Texture> {
        fn texture_inner(
            sel: &mut GfxContext,
            p: PathBuf,
            label: Option<&'static str>,
        ) -> Arc<Texture> {
            if let Some(tex) = sel.textures.get(&p) {
                return tex.clone();
            }
            let tex = Arc::new(Texture::from_path(sel, &p, label));
            sel.textures.insert(p, tex.clone());
            tex
        }

        texture_inner(self, path.into(), label)
    }

    pub fn read_texture(&self, path: impl Into<PathBuf>) -> Option<&Arc<Texture>> {
        self.textures.get(&path.into())
    }

    pub fn palette(&self) -> Arc<Texture> {
        self.textures
            .get(&*PathBuf::from("assets/palette.png"))
            .expect("palette not loaded")
            .clone()
    }

    pub fn set_present_mode(&mut self, mode: wgpu::PresentMode) {
        if self.sc_desc.present_mode != mode {
            self.sc_desc.present_mode = mode;
            self.update_sc = true;
        }
    }

    pub fn set_time(&mut self, time: f32) {
        *self.time_uni.value_mut() = time;
    }

    pub fn set_proj(&mut self, proj: mint::ColumnMatrix4<f32>) {
        *self.projection.value_mut() = proj;
    }

    pub fn set_inv_proj(&mut self, proj: mint::ColumnMatrix4<f32>) {
        *self.inv_projection.value_mut() = proj;
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
        self.light_params.upload_to_gpu(&self.queue);

        encoder
    }

    pub fn render_objs(
        &mut self,
        encoder: &mut CommandEncoder,
        mut prepare: impl for<'a> FnMut(&'a mut FrameContext),
    ) {
        let mut objs = vec![];

        let mut fc = FrameContext {
            objs: &mut objs,
            gfx: self,
        };

        prepare(&mut fc);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &self.color_texture.multisampled_buffer,
                resolve_target: Some(&self.color_texture.target.view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.32,
                        g: 0.63,
                        b: 0.9,
                        a: 0.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: true,
                }),
            }),
        });

        for obj in &mut objs {
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
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &self.ui_texture.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
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

        let pipeline = &self.get_pipeline::<BlitLinear>();
        let bg = self
            .ui_texture
            .bindgroup(&self.device, &pipeline.get_bind_group_layout(0));

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
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

        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &bg, &[]);
        rpass.set_vertex_buffer(0, self.screen_uv_vertices.slice(..));
        rpass.set_index_buffer(self.rect_indices.slice(..), IndexFormat::Uint32);
        rpass.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..1);
    }

    pub fn finish_frame(&mut self, encoder: CommandEncoder) {
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn create_textures(
        device: &Device,
        surface: &Surface,
        desc: &SwapChainDescriptor,
        samples: u32,
    ) -> (SwapChain, Texture, Texture, MultisampledTexture, Texture) {
        (
            device.create_swap_chain(surface, desc),
            Texture::create_depth_texture(device, desc, samples),
            Texture::create_light_texture(device, desc),
            Texture::create_color_texture(device, desc, samples),
            Texture::create_ui_texture(device, desc),
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        self.sc_desc.width = self.size.0;
        self.sc_desc.height = self.size.1;

        let (swapchain, depth, light, color, ui) =
            Self::create_textures(&self.device, &self.surface, &self.sc_desc, self.samples);

        self.swapchain = swapchain;
        self.depth_texture = depth;
        self.light_texture = light;
        self.color_texture = color;
        self.ui_texture = ui;
    }

    pub fn basic_pipeline(
        &self,
        layouts: &[&BindGroupLayout],
        vertex_buffers: &[VertexBufferLayout],
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

        let color_states = [wgpu::ColorTargetState {
            format: self.color_texture.target.format,
            color_blend: wgpu::BlendState {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha_blend: wgpu::BlendState::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader.0,
                entry_point: "main",
                buffers: vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag_shader.0,
                entry_point: "main",
                targets: &color_states,
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: CullMode::Back,
                front_face: FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            multisample: MultisampleState {
                count: self.samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };
        self.device.create_render_pipeline(&render_pipeline_desc)
    }

    pub fn depth_pipeline(
        &self,
        vertex_buffers: &[VertexBufferLayout],
        vert_shader: CompiledShader,
    ) -> RenderPipeline {
        assert!(matches!(vert_shader.1, ShaderType::Vertex));

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("depth pipeline"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert_shader.0,
                entry_point: "main",
                buffers: vertex_buffers,
            },
            fragment: None,
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: CullMode::Back,
                front_face: FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            multisample: MultisampleState {
                count: self.samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };
        self.device.create_render_pipeline(&render_pipeline_desc)
    }

    pub fn get_pipeline<T: 'static>(&self) -> &RenderPipeline {
        &self
            .pipelines
            .get(&std::any::TypeId::of::<T>())
            .expect("Pipeline was not registered in context")
    }

    pub fn register_pipeline<T: 'static>(&mut self, pipe: RenderPipeline) {
        self.pipelines.insert(std::any::TypeId::of::<T>(), pipe);
    }
}

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
