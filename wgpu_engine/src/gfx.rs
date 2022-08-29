use crate::terrain::TerrainPrepared;
use crate::wgpu::SamplerBindingType;
use crate::{
    bg_layout_litmesh, compile_shader, CompiledModule, Drawable, IndexType, InstancedMesh, Mesh,
    SpriteBatch, Texture, TextureBuilder, Uniform, UvVertex, VBDesc,
};
use common::FastMap;
use geom::{vec2, LinearColor, Matrix4, Vec2, Vec3};
use raw_window_handle::HasRawWindowHandle;
use std::any::TypeId;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, BindGroupLayout, BindGroupLayoutDescriptor, BlendComponent, BlendState, CommandBuffer,
    CommandEncoder, CommandEncoderDescriptor, DepthBiasState, Device, ErrorFilter, Face, FrontFace,
    IndexFormat, MultisampleState, PrimitiveState, Queue, RenderPipeline, Surface,
    SurfaceConfiguration, TextureFormat, TextureSampleType, TextureUsages, TextureView,
    VertexBufferLayout,
};

pub struct FBOs {
    pub(crate) depth: Texture,
    pub(crate) color_msaa: TextureView,
    pub(crate) ssao: Texture,
    pub format: TextureFormat,
}

pub struct GfxContext {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub fbos: FBOs,
    pub size: (u32, u32),
    pub(crate) sc_desc: SurfaceConfiguration,
    pub update_sc: bool,
    pub(crate) pipelines: FastMap<TypeId, RenderPipeline>,
    #[allow(clippy::type_complexity)]
    pub(crate) pipelines_builders: Vec<(
        TypeId,
        Vec<String>,
        Box<dyn for<'a> Fn(Vec<CompiledModule>, &'a GfxContext) -> RenderPipeline>,
    )>,
    pub(crate) shader_cache: FastMap<String, CompiledModule>,
    pub(crate) shader_watcher: FastMap<String, (Vec<String>, Option<SystemTime>)>,
    pub(crate) tick: u64,
    pub(crate) projection: Uniform<Matrix4>,
    pub(crate) sun_projection: Uniform<Matrix4>,
    pub render_params: Uniform<RenderParams>,
    pub(crate) textures: FastMap<PathBuf, Arc<Texture>>,
    pub(crate) samples: u32,
    pub(crate) screen_uv_vertices: wgpu::Buffer,
    pub(crate) rect_indices: wgpu::Buffer,
    pub sun_shadowmap: Texture,
    pub simplelit_bg: wgpu::BindGroup,
    pub bnoise_bg: wgpu::BindGroup,
    pub sky_bg: wgpu::BindGroup,
    #[allow(dead_code)] // keep adapter alive
    pub(crate) adapter: Adapter,
}

pub struct Encoders {
    pub smap: Option<CommandBuffer>,
    pub depth_prepass: Option<CommandBuffer>,
    pub end: CommandEncoder,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct RenderParams {
    pub inv_proj: Matrix4,
    pub sun_shadow_proj: Matrix4,
    pub cam_pos: Vec3,
    pub _pad: f32,
    pub cam_dir: Vec3,
    pub _pad4: f32,
    pub sun: Vec3,
    pub _pad2: f32,
    pub sun_col: LinearColor,
    pub grass_col: LinearColor,
    pub sand_col: LinearColor,
    pub sea_col: LinearColor,
    pub viewport: Vec2,
    pub time: f32,
    pub ssao_strength: f32,
    pub ssao_radius: f32,
    pub ssao_falloff: f32,
    pub ssao_base: f32,
    pub ssao_samples: i32,
    pub ssao_enabled: i32,
    pub shadow_mapping_enabled: i32,
    pub realistic_sky: i32,
    pub grid_enabled: i32,
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            inv_proj: Matrix4::zero(),
            sun_shadow_proj: Matrix4::zero(),
            sun_col: Default::default(),
            grass_col: Default::default(),
            sand_col: Default::default(),
            sea_col: Default::default(),
            cam_pos: Default::default(),
            cam_dir: Default::default(),
            sun: Default::default(),
            viewport: vec2(1000.0, 1000.0),
            time: 0.0,
            ssao_strength: 0.0,
            ssao_radius: 0.0,
            ssao_falloff: 0.0,
            ssao_base: 0.0,
            ssao_samples: 0,
            ssao_enabled: 1,
            shadow_mapping_enabled: 1,
            realistic_sky: 1,
            _pad: 0.0,
            _pad2: 0.0,
            _pad4: 0.0,
            grid_enabled: 1,
        }
    }
}

u8slice_impl!(RenderParams);

pub struct GuiRenderContext<'a, 'b> {
    pub encoder: &'a mut CommandEncoder,
    pub view: &'a TextureView,
    pub size: (u32, u32),
    pub device: &'a Device,
    pub queue: &'a Queue,
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
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect(
                "failed to find a suitable adapter, have you installed necessary vulkan libraries?",
            );

        let limit = if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: limit,
                },
                None,
            )
            .await
            .expect("could not find device, have you installed necessary vulkan libraries?");

        let formats = surface.get_supported_formats(&adapter);

        let format = *formats
            .iter()
            .filter(|x| x.describe().srgb)
            .next()
            .unwrap_or_else(|| &formats[0]);

        let sc_desc = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: win_width,
            height: win_height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        let samples = if cfg!(target_arch = "wasm32") { 1 } else { 4 };
        let fbos = Self::create_textures(&device, &sc_desc, samples);
        surface.configure(&device, &sc_desc);

        let projection = Uniform::new(Matrix4::zero(), &device);

        let screen_uv_vertices = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(SCREEN_UV_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let rect_indices = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let blue_noise = TextureBuilder::from_path("assets/sprites/blue_noise_512.png")
            .with_label("blue noise")
            .with_srgb(false)
            .with_mipmaps(false)
            .with_sampler(Texture::nearest_sampler())
            .build(&device, &queue);

        let bnoise_bg = blue_noise.bindgroup(&device, &Texture::bindgroup_layout(&device));

        let mut textures = FastMap::default();
        textures.insert(
            PathBuf::from("assets/sprites/blue_noise_512.png"),
            Arc::new(blue_noise),
        );

        let mut me = Self {
            size: (win_width, win_height),
            queue,
            sc_desc,
            update_sc: false,
            adapter,
            fbos,
            surface,
            pipelines: FastMap::default(),
            pipelines_builders: vec![],
            shader_cache: Default::default(),
            shader_watcher: Default::default(),
            tick: 0,
            projection,
            sun_projection: Uniform::new(Matrix4::zero(), &device),
            render_params: Uniform::new(Default::default(), &device),
            textures,
            samples,
            screen_uv_vertices,
            rect_indices,
            simplelit_bg: Uniform::new([0.0f32; 4], &device).bindgroup, // bogus
            sky_bg: Uniform::new([0.0f32; 4], &device).bindgroup,       // bogus
            bnoise_bg,
            sun_shadowmap: Self::mk_shadowmap(&device, 2048),
            device,
        };

        me.update_simplelit_bg();

        TerrainPrepared::setup(&mut me);
        Mesh::setup(&mut me);
        InstancedMesh::setup(&mut me);
        SpriteBatch::setup(&mut me);
        SSAOPipeline::setup(&mut me);
        BackgroundPipeline::setup(&mut me);

        let p = TextureBuilder::from_path("assets/sprites/palette.png")
            .with_label("palette")
            .with_sampler(Texture::nearest_sampler())
            .build(&me.device, &me.queue);
        me.set_texture("assets/sprites/palette.png", p);

        let gs = me.texture("assets/sprites/gradientsky.png", "gradient sky");
        let starfield = me.texture("assets/sprites/starfield.png", "starfield");

        me.sky_bg = Texture::multi_bindgroup(
            &[&*gs, &*starfield],
            &me.device,
            &Texture::bindgroup_layout_complex(
                &me.device,
                TextureSampleType::Float { filterable: true },
                2,
                false,
            ),
        );

        me
    }

    pub fn mk_shadowmap(device: &Device, res: u32) -> Texture {
        let mut smap = Texture::create_depth_texture(device, (res, res), 1);
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
            .get(&*PathBuf::from("assets/sprites/palette.png"))
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

    pub fn start_frame(&mut self) -> Encoders {
        let end = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("End encoder"),
            });

        *self.sun_projection.value_mut() = self.render_params.value().sun_shadow_proj;

        self.projection.upload_to_gpu(&self.queue);
        self.sun_projection.upload_to_gpu(&self.queue);

        Encoders {
            smap: None,
            depth_prepass: None,
            end,
        }
    }

    #[profiling::function]
    pub fn render_objs(
        &mut self,
        encs: &mut Encoders,
        frame: &TextureView,
        mut prepare: impl FnMut(&mut FrameContext<'_>),
    ) {
        let mut objs = vec![];
        let mut fc = FrameContext {
            objs: &mut objs,
            gfx: self,
        };

        prepare(&mut fc);

        let objsref = &*objs;
        let enc_dep_ext = &mut encs.depth_prepass;
        let enc_smap_ext = &mut encs.smap;

        profiling::scope!("depth prepass");
        let mut prepass = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("depth prepass encoder"),
            });

        {
            let mut depth_prepass = prepass.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.fbos.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for obj in objsref.iter() {
                obj.draw_depth(self, &mut depth_prepass, false, &self.projection.bindgroup);
            }
            drop(depth_prepass);
        }
        *enc_dep_ext = Some(prepass.finish());
        if self.render_params.value().shadow_mapping_enabled != 0 {
            profiling::scope!("shadow pass");
            let mut smap_enc = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("shadow map encoder"),
                });
            let mut sun_shadow_pass = smap_enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.sun_shadowmap.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for obj in objsref.iter() {
                obj.draw_depth(
                    self,
                    &mut sun_shadow_pass,
                    true,
                    &self.sun_projection.bindgroup,
                );
            }
            drop(sun_shadow_pass);
            *enc_smap_ext = Some(smap_enc.finish());
        }

        if self.render_params.value().ssao_enabled != 0 {
            profiling::scope!("ssao");
            let pipeline = self.get_pipeline::<SSAOPipeline>();
            let bg = self
                .fbos
                .depth
                .bindgroup(&self.device, &pipeline.get_bind_group_layout(0));

            let mut ssao_pass = encs.end.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.fbos.ssao.view,
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
                })],
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
            profiling::scope!("main render pass");
            let mut render_pass = encs.end.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.fbos.color_msaa,
                    resolve_target: Some(frame),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.fbos.depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            for obj in objsref.iter() {
                obj.draw(self, &mut render_pass);
            }
        }

        {
            profiling::scope!("bg pass");
            let mut bg_pass = encs.end.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.fbos.color_msaa,
                    resolve_target: Some(frame),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.fbos.depth.view,
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
            bg_pass.set_bind_group(2, &self.sky_bg, &[]);
            bg_pass.set_vertex_buffer(0, self.screen_uv_vertices.slice(..));
            bg_pass.set_index_buffer(self.rect_indices.slice(..), IndexFormat::Uint32);
            bg_pass.draw_indexed(0..6, 0, 0..1);
        }
    }

    #[profiling::function]
    pub fn render_gui(
        &mut self,
        encoders: &mut Encoders,
        frame: &TextureView,
        mut render_gui: impl FnMut(GuiRenderContext<'_, '_>),
    ) {
        render_gui(GuiRenderContext {
            encoder: &mut encoders.end,
            view: frame,
            size: self.size,
            device: &self.device,
            queue: &self.queue,
            rpass: None,
        });
    }

    pub fn finish_frame(&mut self, encoder: Encoders) {
        self.queue.submit(
            encoder
                .depth_prepass
                .into_iter()
                .chain(encoder.smap)
                .chain(Some(encoder.end.finish())),
        );
        #[cfg(debug_assertions)]
        self.check_shader_updates();
        self.tick += 1;
    }

    pub fn create_textures(device: &Device, desc: &SurfaceConfiguration, samples: u32) -> FBOs {
        let size = (desc.width, desc.height);
        let ssao = Texture::create_fbo(
            device,
            size,
            wgpu::TextureFormat::R8Unorm,
            TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            None,
        );
        FBOs {
            depth: Texture::create_depth_texture(device, size, samples),
            color_msaa: Texture::create_color_msaa(device, desc, samples),
            ssao,
            format: desc.format,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.size = (width, height);
        self.sc_desc.width = self.size.0;
        self.sc_desc.height = self.size.1;

        self.surface.configure(&self.device, &self.sc_desc);
        self.fbos = Self::create_textures(&self.device, &self.sc_desc, self.samples);
        self.update_simplelit_bg();
    }

    pub fn update_simplelit_bg(&mut self) {
        self.simplelit_bg = Texture::multi_bindgroup(
            &[
                &self.fbos.ssao,
                self.read_texture("assets/sprites/blue_noise_512.png")
                    .expect("blue noise not initialized"),
                &self.sun_shadowmap,
            ],
            &self.device,
            &bg_layout_litmesh(&self.device),
        );
    }

    pub fn color_pipeline(
        &self,
        label: &'static str,
        layouts: &[&BindGroupLayout],
        vertex_buffers: &[VertexBufferLayout<'_>],
        vert_shader: &CompiledModule,
        frag_shader: &CompiledModule,
    ) -> RenderPipeline {
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("color pipeline"),
                    bind_group_layouts: layouts,
                    push_constant_ranges: &[],
                });

        let color_states = [Some(wgpu::ColorTargetState {
            format: self.sc_desc.format,
            blend: Some(BlendState {
                color: BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: BlendComponent::REPLACE,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: vert_shader,
                entry_point: "vert",
                buffers: vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: frag_shader,
                entry_point: "frag",
                targets: &color_states,
            }),
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                front_face: FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: MultisampleState {
                count: self.samples,
                ..Default::default()
            },
            multiview: None,
        };
        self.device.create_render_pipeline(&render_pipeline_desc)
    }

    pub fn depth_pipeline(
        &self,
        vertex_buffers: &[VertexBufferLayout<'_>],
        vert_shader: &CompiledModule,
        shadow_map: bool,
    ) -> RenderPipeline {
        self.depth_pipeline_bglayout(
            vertex_buffers,
            vert_shader,
            shadow_map,
            &[&self.projection.layout],
        )
    }

    pub fn depth_pipeline_bglayout(
        &self,
        vertex_buffers: &[VertexBufferLayout<'_>],
        vert_shader: &CompiledModule,
        shadow_map: bool,
        layouts: &[&BindGroupLayout],
    ) -> RenderPipeline {
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("depth pipeline"),
                    bind_group_layouts: layouts,
                    push_constant_ranges: &[],
                });

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: vert_shader,
                entry_point: "vert",
                buffers: vertex_buffers,
            },
            fragment: None,
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(Face::Back),
                front_face: FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: if shadow_map {
                    wgpu::CompareFunction::LessEqual
                } else {
                    wgpu::CompareFunction::GreaterEqual
                },
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
            }),
            multisample: MultisampleState {
                count: if shadow_map { 1 } else { self.samples },
                ..Default::default()
            },
            multiview: None,
        };
        self.device.create_render_pipeline(&render_pipeline_desc)
    }

    pub fn get_pipeline<T: 'static>(&self) -> &RenderPipeline {
        self.pipelines
            .get(&TypeId::of::<T>())
            .expect("Pipeline was not registered in context")
    }

    pub fn get_modules<'a>(
        &'a mut self,
        shaders: impl Iterator<Item = String> + 'a,
    ) -> impl Iterator<Item = CompiledModule> + 'a {
        shaders.map(move |shader| {
            let device = &self.device;
            self.shader_cache
                .entry(shader)
                .or_insert_with_key(move |key| compile_shader(device, key))
                .clone()
        })
    }

    pub fn check_shader_updates(&mut self) {
        if self.tick % 30 != 0 {
            return;
        }
        let mut to_invalidate = vec![];
        for (sname, (parents, entry)) in &mut self.shader_watcher {
            let meta =
                unwrap_cont!(
                    std::fs::metadata(Path::new(&format!("assets/shaders/{}", sname))).ok()
                );
            let filetime = unwrap_cont!(meta.modified().ok());
            match entry.as_mut() {
                Some(entry) => {
                    if *entry < filetime {
                        to_invalidate.push(sname.clone());
                        to_invalidate.extend(parents.iter().cloned());
                        *entry = filetime;
                    }
                }
                None => {
                    *entry = Some(filetime);
                }
            }
        }
        for sname in to_invalidate {
            self.invalidate(&sname);
        }
    }

    pub fn invalidate(&mut self, shader_name: &str) {
        if let Some(x) = self.shader_cache.get_mut(shader_name) {
            let device = &self.device;
            device.push_error_scope(ErrorFilter::Validation);
            let new_shader = compile_shader(device, shader_name);
            let scope = futures::executor::block_on(device.pop_error_scope());
            if scope.is_some() {
                return;
            }
            *x = new_shader;
        } else {
            return;
        }
        for (ty, deps, pipe) in &self.pipelines_builders {
            if deps.iter().all(|x| x != shader_name) {
                continue;
            }
            let shader_cache = &mut self.shader_cache;
            let device = &self.device;
            let modules = deps
                .iter()
                .map(|x| x.to_string())
                .map(move |shader| {
                    shader_cache
                        .entry(shader)
                        .or_insert_with_key(move |key| compile_shader(device, key))
                        .clone()
                })
                .collect();
            if self.pipelines.contains_key(ty) {
                let pipeline = pipe(modules, self);
                self.pipelines.insert(*ty, pipeline);
            }
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn register_pipeline<T: 'static>(
        &mut self,
        shaders: &[&str],
        pipe: Box<dyn for<'a, 'b> Fn(Vec<CompiledModule>, &'a Self) -> RenderPipeline>,
    ) {
        let shaders: Vec<_> = shaders.iter().map(|x| x.to_string() + ".wgsl").collect();

        let modules: Vec<_> = self
            .get_modules(shaders.iter().map(|s| s.to_string()))
            .collect();

        #[cfg(debug_assertions)]
        for (sname, module) in shaders.iter().zip(&modules) {
            for dep in module.get_deps() {
                self.shader_watcher
                    .entry(dep.to_string())
                    .or_insert((vec![], None))
                    .0
                    .push(sname.to_string());
            }
        }

        let pipeline = pipe(modules, self);

        #[cfg(debug_assertions)]
        self.pipelines_builders
            .push((TypeId::of::<T>(), shaders, pipe));
        if self.pipelines.insert(TypeId::of::<T>(), pipeline).is_some() {
            log::error!(
                "pipeline for same type inserted registered multiple times! {:?}",
                TypeId::of::<T>()
            );
        }
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
                                        visibility: wgpu::ShaderStages::FRAGMENT,
                                        ty: wgpu::BindingType::Texture {
                                            multisampled: cfg!(not(target_arch = "wasm32")),
                                            view_dimension: wgpu::TextureViewDimension::D2,
                                            sample_type: TextureSampleType::Float {
                                                filterable: true,
                                            },
                                        },
                                        count: None,
                                    },
                                    wgpu::BindGroupLayoutEntry {
                                        binding: 1,
                                        visibility: wgpu::ShaderStages::FRAGMENT,
                                        ty: wgpu::BindingType::Sampler(
                                            SamplerBindingType::Filtering,
                                        ),
                                        count: None,
                                    },
                                ],
                            }),
                        &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                    ],
                    push_constant_ranges: &[],
                });

        let color_states = [Some(wgpu::ColorTargetState {
            format: gfx.fbos.ssao.format,
            write_mask: wgpu::ColorWrites::ALL,
            blend: Some(BlendState {
                color: BlendComponent::REPLACE,
                alpha: BlendComponent::REPLACE,
            }),
        })];

        gfx.register_pipeline::<SSAOPipeline>(
            &["ssao"],
            Box::new(move |m, gfx| {
                let ssao = &m[0];

                let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: ssao,
                        entry_point: "vert",
                        buffers: &[UvVertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: ssao,
                        entry_point: "frag",
                        targets: &color_states,
                    }),
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    multiview: None,
                };

                gfx.device.create_render_pipeline(&render_pipeline_desc)
            }),
        );
    }
}

struct BackgroundPipeline;

impl BackgroundPipeline {
    pub fn setup(gfx: &mut GfxContext) {
        gfx.register_pipeline::<BackgroundPipeline>(
            &["background"],
            Box::new(move |m, gfx| {
                let bg = &m[0];
                gfx.color_pipeline(
                    "background",
                    &[
                        &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                        &Texture::bindgroup_layout(&gfx.device),
                        &Texture::bindgroup_layout_complex(
                            &gfx.device,
                            TextureSampleType::Float { filterable: true },
                            2,
                            false,
                        ),
                    ],
                    &[UvVertex::desc()],
                    bg,
                    bg,
                )
            }),
        );
    }
}
