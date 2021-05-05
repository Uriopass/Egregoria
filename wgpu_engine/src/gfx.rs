use crate::ShaderType;
use crate::{
    bg_layout_litmesh, compile_shader, BlitLinear, CompiledShader, Drawable, IndexType,
    InstancedMesh, Mesh, SpriteBatch, Texture, TextureBuilder, Uniform, UvVertex, VBDesc,
};
use common::FastMap;
use geom::{vec2, LinearColor, Matrix4, Vec2, Vec3};
use raw_window_handle::HasRawWindowHandle;
use std::any::TypeId;
use std::path::PathBuf;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, BindGroupLayout, BindGroupLayoutDescriptor, CommandEncoder, CommandEncoderDescriptor,
    CullMode, DepthBiasState, Device, FrontFace, IndexFormat, MultisampleState, PrimitiveState,
    Queue, RenderPipeline, Surface, SwapChain, SwapChainDescriptor, SwapChainFrame,
    TextureSampleType, TextureUsage, VertexBufferLayout,
};

pub struct FBOs {
    pub swapchain: SwapChain,
    pub(crate) depth: Texture,
    pub(crate) light: Texture,
    pub(crate) color_msaa: wgpu::TextureView,
    pub(crate) ui: Texture,
    pub(crate) ssao: Texture,
}

pub struct GfxContext {
    pub(crate) surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub fbos: FBOs,
    pub size: (u32, u32),
    pub(crate) sc_desc: SwapChainDescriptor,
    pub update_sc: bool,
    pub(crate) pipelines: FastMap<TypeId, RenderPipeline>,
    pub(crate) projection: Uniform<Matrix4>,
    pub render_params: Uniform<RenderParams>,
    pub(crate) textures: FastMap<PathBuf, Arc<Texture>>,
    pub(crate) samples: u32,
    pub(crate) screen_uv_vertices: wgpu::Buffer,
    pub(crate) rect_indices: wgpu::Buffer,
    pub sun_shadowmap: Texture,
    pub simplelit_bg: wgpu::BindGroup,
    pub bnoise_bg: wgpu::BindGroup,
    #[allow(dead_code)] // keep adapter alive
    pub(crate) adapter: Adapter,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct RenderParams {
    pub inv_proj: Matrix4,
    pub sun_shadow_proj: Matrix4,
    pub cam_pos: Vec3,
    pub _pad: f32,
    pub sun: Vec3,
    pub _pad2: f32,
    pub sun_col: LinearColor,
    pub viewport: Vec2,
    pub time: f32,
    pub ssao_strength: f32,
    pub ssao_radius: f32,
    pub ssao_falloff: f32,
    pub ssao_base: f32,
    pub ssao_samples: i32,
    pub ssao_enabled: i32,
    pub shadow_mapping_enabled: i32,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            inv_proj: Matrix4::from([0.0; 16]),
            sun_shadow_proj: Matrix4::from([0.0; 16]),
            sun_col: Default::default(),
            cam_pos: Default::default(),
            _pad: 0.0,
            sun: Default::default(),
            _pad2: 0.0,
            viewport: vec2(1000.0, 1000.0),
            time: 0.0,
            ssao_strength: 0.0,
            ssao_radius: 0.0,
            ssao_falloff: 0.0,
            ssao_base: 0.0,
            ssao_samples: 0,
            ssao_enabled: 1,
            shadow_mapping_enabled: 1,
        }
    }
}

u8slice_impl!(RenderParams);

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
                    features: wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
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
        let fbos = Self::create_textures(&device, &surface, &sc_desc, samples);

        let projection = Uniform::new(Matrix4::from([0.0; 16]), &device);

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

        let blue_noise = TextureBuilder::from_path("assets/blue_noise_512.png")
            .with_label("blue noise")
            .with_srgb(false)
            .with_mipmaps(false)
            .with_sampler(Texture::nearest_sampler())
            .build(&device, &queue);

        let bnoise_bg = blue_noise.bindgroup(&device, &Texture::bindgroup_layout(&device));

        let mut textures = FastMap::default();
        textures.insert(
            PathBuf::from("assets/blue_noise_512.png"),
            Arc::new(blue_noise),
        );
        let bogus_bindgroup = Uniform::new([0.0f32; 4], &device).bindgroup;

        let mut me = Self {
            size: (win_width, win_height),
            queue,
            sc_desc,
            update_sc: false,
            adapter,
            fbos,
            surface,
            pipelines: FastMap::default(),
            projection,
            render_params: Uniform::new(Default::default(), &device),
            textures,
            samples,
            screen_uv_vertices,
            rect_indices,
            simplelit_bg: bogus_bindgroup,
            bnoise_bg,
            sun_shadowmap: Self::mk_shadowmap(&device, 2048),
            device,
        };

        me.update_simplelit_bg();

        Mesh::setup(&mut me);
        InstancedMesh::setup(&mut me);
        SpriteBatch::setup(&mut me);
        crate::lighting::setup(&mut me);
        BlitLinear::setup(&mut me);
        SSAOPipeline::setup(&mut me);
        BackgroundPipeline::setup(&mut me);

        let p = TextureBuilder::from_path("assets/palette.png")
            .with_label("palette")
            .with_sampler(Texture::nearest_sampler())
            .build(&me.device, &me.queue);
        me.set_texture("assets/palette.png", p);

        me
    }

    pub fn mk_shadowmap(device: &Device, res: u32) -> Texture {
        let mut smap = Texture::create_depth_texture(&device, (res, res), 1);
        smap.sampler = device.create_sampler(&Texture::depth_compare_sampler());
        smap
    }

    pub fn set_texture(&mut self, path: impl Into<PathBuf>, tex: Texture) {
        let p = path.into();
        self.textures.insert(p, Arc::new(tex));
    }

    pub fn texture(&mut self, path: impl Into<PathBuf>, label: &'static str) -> Arc<Texture> {
        self.texture_inner(path.into(), label)
            .expect("tex not found")
    }

    pub fn try_texture(
        &mut self,
        path: impl Into<PathBuf>,
        label: &'static str,
    ) -> Option<Arc<Texture>> {
        self.texture_inner(path.into(), label)
    }

    fn texture_inner(&mut self, p: PathBuf, label: &'static str) -> Option<Arc<Texture>> {
        if let Some(tex) = self.textures.get(&p) {
            return Some(tex.clone());
        }
        let tex = Arc::new(
            TextureBuilder::try_from_path(&p)?
                .with_label(label)
                .build(&self.device, &self.queue),
        );
        self.textures.insert(p, tex.clone());
        Some(tex)
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
        self.render_params.value_mut().time = time;
    }

    pub fn set_proj(&mut self, proj: Matrix4) {
        *self.projection.value_mut() = proj;
    }

    pub fn set_inv_proj(&mut self, proj: Matrix4) {
        self.render_params.value_mut().inv_proj = proj;
    }

    pub fn start_frame(&mut self) -> CommandEncoder {
        let encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        self.projection.upload_to_gpu(&self.queue);

        encoder
    }

    pub fn render_objs(
        &mut self,
        encoder: &mut CommandEncoder,
        frame: &SwapChainFrame,
        mut prepare: impl FnMut(&mut FrameContext),
    ) {
        let mut objs = vec![];

        let mut fc = FrameContext {
            objs: &mut objs,
            gfx: self,
        };

        prepare(&mut fc);

        {
            let mut depth_prepass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.fbos.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for obj in &mut objs {
                obj.draw_depth(&self, &mut depth_prepass);
            }
        }

        let prev = std::mem::replace(
            &mut self.projection,
            Uniform::new(self.render_params.value().sun_shadow_proj, &self.device),
        );
        let prevs = self.samples;
        self.samples = 1;
        if self.render_params.value().shadow_mapping_enabled != 0 {
            let mut sun_shadow_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.sun_shadowmap.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for obj in &mut objs {
                obj.draw_depth(&self, &mut sun_shadow_pass);
            }
        }
        self.projection = prev;
        self.samples = prevs;

        if self.render_params.value().ssao_enabled != 0 {
            let pipeline = self.get_pipeline::<SSAOPipeline>();
            let bg = self
                .fbos
                .depth
                .bindgroup(&self.device, &pipeline.get_bind_group_layout(0));

            let mut ssao_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.fbos.ssao.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            ssao_pass.set_pipeline(pipeline);
            ssao_pass.set_bind_group(0, &bg, &[]);
            ssao_pass.set_bind_group(1, &self.render_params.bindgroup, &[]);
            ssao_pass.set_vertex_buffer(0, self.screen_uv_vertices.slice(..));
            ssao_pass.set_index_buffer(self.rect_indices.slice(..), IndexFormat::Uint32);
            ssao_pass.draw_indexed(0..6, 0, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.fbos.color_msaa,
                    resolve_target: Some(&frame.output.view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.fbos.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for obj in &mut objs {
                obj.draw(&self, &mut render_pass);
            }
        }

        {
            let mut bg_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &self.fbos.color_msaa,
                    resolve_target: Some(&frame.output.view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.fbos.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            bg_pass.set_pipeline(self.get_pipeline::<BackgroundPipeline>());
            bg_pass.set_bind_group(0, &self.render_params.bindgroup, &[]);
            bg_pass.set_bind_group(1, &self.bnoise_bg, &[]);
            bg_pass.set_vertex_buffer(0, self.screen_uv_vertices.slice(..));
            bg_pass.set_index_buffer(self.rect_indices.slice(..), IndexFormat::Uint32);
            bg_pass.draw_indexed(0..6, 0, 0..1);
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
                attachment: &self.fbos.ui.view,
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
            .fbos
            .ui
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
    ) -> FBOs {
        let size = (desc.width, desc.height);
        let ssao = Texture::create_fbo(
            device,
            size,
            wgpu::TextureFormat::R8Unorm,
            TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_SRC,
            None,
        );
        FBOs {
            swapchain: device.create_swap_chain(surface, desc),
            depth: Texture::create_depth_texture(device, size, samples),
            light: Texture::create_light_texture(device, desc),
            color_msaa: Texture::create_color_msaa(device, desc, samples),
            ui: Texture::create_ui_texture(device, desc),
            ssao,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        self.sc_desc.width = self.size.0;
        self.sc_desc.height = self.size.1;

        self.fbos = Self::create_textures(&self.device, &self.surface, &self.sc_desc, self.samples);
        self.update_simplelit_bg();
    }

    pub fn update_simplelit_bg(&mut self) {
        self.simplelit_bg = Texture::multi_bindgroup(
            &[
                &self.fbos.ssao,
                &self
                    .read_texture("assets/blue_noise_512.png")
                    .expect("blue noise not initialized"),
                &self.sun_shadowmap,
                &self.fbos.light,
            ],
            &self.device,
            &bg_layout_litmesh(&self.device),
        );
    }

    pub fn color_pipeline(
        &self,
        layouts: &[&BindGroupLayout],
        vertex_buffers: &[VertexBufferLayout],
        vert_shader: &CompiledShader,
        frag_shader: &CompiledShader,
    ) -> RenderPipeline {
        assert!(matches!(vert_shader.1, ShaderType::Vertex));
        assert!(matches!(frag_shader.1, ShaderType::Fragment));

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("color pipeline"),
                    bind_group_layouts: layouts,
                    push_constant_ranges: &[],
                });

        let color_states = [wgpu::ColorTargetState {
            format: self.sc_desc.format,
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
                cull_mode: CullMode::Back,
                front_face: FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
                clamp_depth: false,
            }),
            multisample: MultisampleState {
                count: self.samples,
                ..Default::default()
            },
        };
        self.device.create_render_pipeline(&render_pipeline_desc)
    }

    pub fn depth_pipeline(
        &self,
        vertex_buffers: &[VertexBufferLayout],
        vert_shader: &CompiledShader,
        shadow_map: bool,
    ) -> RenderPipeline {
        assert!(matches!(vert_shader.1, ShaderType::Vertex));

        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("depth pipeline"),
                    bind_group_layouts: &[&self.projection.layout],
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
                bias: if shadow_map {
                    DepthBiasState {
                        constant: 1,
                        slope_scale: 1.75,
                        clamp: 0.0,
                    }
                } else {
                    Default::default()
                },
                clamp_depth: false,
            }),
            multisample: MultisampleState {
                count: if shadow_map { 1 } else { self.samples },
                ..Default::default()
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

struct SSAOPipeline;

impl SSAOPipeline {
    pub fn setup(gfx: &mut GfxContext) {
        let blit_linear = compile_shader(&gfx.device, "assets/shaders/blit_linear.vert", None);
        let ssao_frag = compile_shader(&gfx.device, "assets/shaders/ssao.frag", None);
        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("ssao pipeline"),
                    bind_group_layouts: &[
                        &gfx.device
                            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                                label: Some("ssao depth bg layout"),
                                entries: &[
                                    wgpu::BindGroupLayoutEntry {
                                        binding: 0,
                                        visibility: wgpu::ShaderStage::FRAGMENT,
                                        ty: wgpu::BindingType::Texture {
                                            multisampled: true,
                                            view_dimension: wgpu::TextureViewDimension::D2,
                                            sample_type: TextureSampleType::Float {
                                                filterable: true,
                                            },
                                        },
                                        count: None,
                                    },
                                    wgpu::BindGroupLayoutEntry {
                                        binding: 1,
                                        visibility: wgpu::ShaderStage::FRAGMENT,
                                        ty: wgpu::BindingType::Sampler {
                                            filtering: true,
                                            comparison: false,
                                        },
                                        count: None,
                                    },
                                ],
                            }),
                        &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                    ],
                    push_constant_ranges: &[],
                });

        let color_states = [wgpu::ColorTargetState {
            format: gfx.fbos.ssao.format,
            color_blend: wgpu::BlendState::REPLACE,
            alpha_blend: wgpu::BlendState::REPLACE,
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_linear.0,
                entry_point: "main",
                buffers: &[UvVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &ssao_frag.0,
                entry_point: "main",
                targets: &color_states,
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
        };

        gfx.register_pipeline::<SSAOPipeline>(
            gfx.device.create_render_pipeline(&render_pipeline_desc),
        );
    }
}

struct BackgroundPipeline;

impl BackgroundPipeline {
    pub fn setup(gfx: &mut GfxContext) {
        let bg_vert = compile_shader(&gfx.device, "assets/shaders/background.vert", None);
        let bg_frag = compile_shader(&gfx.device, "assets/shaders/background.frag", None);
        let pipe = gfx.color_pipeline(
            &[
                &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                &Texture::bindgroup_layout(&gfx.device),
            ],
            &[UvVertex::desc()],
            &bg_vert,
            &bg_frag,
        );

        gfx.register_pipeline::<BackgroundPipeline>(pipe);
    }
}
