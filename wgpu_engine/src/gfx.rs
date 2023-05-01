use crate::pbr::PBR;
use crate::{
    bg_layout_litmesh, CompiledModule, Drawable, IndexType, LampLights, Material, MaterialID,
    MaterialMap, PipelineBuilder, Pipelines, Texture, TextureBuilder, Uniform, UvVertex, TL,
};
use common::FastMap;
use geom::{vec2, LinearColor, Matrix4, Vec2, Vec3};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use wgpu::util::{backend_bits_from_env, BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, Backends, BindGroupLayout, BlendComponent, BlendState, CommandBuffer, CommandEncoder,
    CommandEncoderDescriptor, CompositeAlphaMode, DepthBiasState, Device, Face, FragmentState,
    FrontFace, IndexFormat, InstanceDescriptor, MultisampleState, PipelineLayoutDescriptor,
    PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Surface, SurfaceConfiguration, SurfaceTexture, TextureAspect,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexBufferLayout, VertexState,
};

pub struct FBOs {
    pub(crate) depth: Texture,
    pub(crate) depth_bg: wgpu::BindGroup,
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

    pub(crate) materials: MaterialMap,
    pub(crate) default_material: Material,
    pub tick: u64,
    pub(crate) pipelines: RefCell<Pipelines>,
    pub(crate) projection: Uniform<Matrix4>,
    pub(crate) sun_projection: [Uniform<Matrix4>; N_CASCADES],
    pub render_params: Uniform<RenderParams>,
    pub(crate) texture_cache_paths: FastMap<PathBuf, Arc<Texture>>,
    pub(crate) texture_cache_bytes: Mutex<HashMap<u64, Arc<Texture>, common::TransparentHasherU64>>,
    pub(crate) samples: u32,
    pub(crate) screen_uv_vertices: wgpu::Buffer,
    pub(crate) rect_indices: wgpu::Buffer,
    pub sun_shadowmap: Texture,
    pub pbr: PBR,
    pub lamplights: LampLights,

    pub simplelit_bg: wgpu::BindGroup,
    pub bnoise_bg: wgpu::BindGroup,
    pub sky_bg: wgpu::BindGroup,
    #[allow(dead_code)] // keep adapter alive
    pub(crate) adapter: Adapter,
}

pub struct Encoders {
    pub pbr: Option<CommandBuffer>,
    pub smap: Option<CommandBuffer>,
    pub depth_prepass: Option<CommandBuffer>,
    pub end: CommandEncoder,
}

pub const N_CASCADES: usize = 4;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct RenderParams {
    pub inv_proj: Matrix4,
    pub sun_shadow_proj: [Matrix4; N_CASCADES],
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
    pub time_always: f32,
    pub ssao_enabled: i32,
    pub shadow_mapping_resolution: i32,
    pub grid_enabled: i32,
    pub _pad5: [f32; 3],
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            inv_proj: Matrix4::zero(),
            sun_shadow_proj: [Matrix4::zero(); N_CASCADES],
            sun_col: Default::default(),
            grass_col: Default::default(),
            sand_col: Default::default(),
            sea_col: Default::default(),
            cam_pos: Default::default(),
            cam_dir: Default::default(),
            sun: Default::default(),
            viewport: vec2(1000.0, 1000.0),
            time: 0.0,
            time_always: 0.0,
            ssao_enabled: 1,
            shadow_mapping_resolution: 2048,
            grid_enabled: 1,
            _pad: 0.0,
            _pad2: 0.0,
            _pad4: 0.0,
            _pad5: Default::default(),
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
    pub async fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window: &W,
        win_width: u32,
        win_height: u32,
    ) -> Self {
        let mut backends = backend_bits_from_env().unwrap_or_else(Backends::all);
        if std::env::var("RENDERDOC").is_ok() {
            backends = Backends::VULKAN;
        }

        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(window).unwrap() };
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

        let capabilities = surface.get_capabilities(&adapter);

        let format = *capabilities
            .formats
            .iter()
            .find(|x| x.is_srgb())
            .unwrap_or_else(|| &capabilities.formats[0]);

        let sc_desc = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: win_width,
            height: win_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
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
            .with_sampler(Texture::nearest_sampler())
            .build(&device, &queue);

        let bnoise_bg =
            blue_noise.bindgroup(&device, &Texture::bindgroup_layout(&device, [TL::Float]));

        let mut textures = FastMap::default();
        textures.insert(
            PathBuf::from("assets/sprites/blue_noise_512.png"),
            Arc::new(blue_noise),
        );

        let pbr = PBR::new(&device, &queue);

        let mut me = Self {
            size: (win_width, win_height),
            sc_desc,
            update_sc: false,
            adapter,
            fbos,
            surface,
            pipelines: RefCell::new(Pipelines::new(&device)),
            materials: Default::default(),
            default_material: Material::new_default(&device, &queue),
            tick: 0,
            projection,
            sun_projection: [(); 4].map(|_| Uniform::new(Matrix4::zero(), &device)),
            render_params: Uniform::new(Default::default(), &device),
            texture_cache_paths: textures,
            texture_cache_bytes: Default::default(),
            samples,
            screen_uv_vertices,
            rect_indices,
            simplelit_bg: Uniform::new([0.0f32; 4], &device).bindgroup, // bogus
            sky_bg: Uniform::new([0.0f32; 4], &device).bindgroup,       // bogus
            bnoise_bg,
            sun_shadowmap: Self::mk_shadowmap(&device, 2048),
            lamplights: LampLights::new(&device, &queue),
            device,
            queue,
            pbr,
        };

        me.update_simplelit_bg();

        let palette = TextureBuilder::from_path("assets/sprites/palette.png")
            .with_label("palette")
            .with_sampler(Texture::nearest_sampler())
            .with_mipmaps(me.mipmap_module())
            .build(&me.device, &me.queue);
        me.set_texture("assets/sprites/palette.png", palette);

        let starfield = me.texture("assets/sprites/starfield.png", "starfield");

        me.sky_bg = Texture::multi_bindgroup(
            &[&*starfield, &me.pbr.environment_cube],
            &me.device,
            &BackgroundPipeline::bglayout_texs(&me),
        );

        me
    }

    pub fn register_material(&mut self, material: Material) -> MaterialID {
        self.materials.insert(material)
    }

    pub fn material(&self, id: MaterialID) -> &Material {
        self.materials.get(id).unwrap_or(&self.default_material)
    }

    pub fn material_mut(&mut self, id: MaterialID) -> Option<(&mut Material, &Queue)> {
        self.materials.get_mut(id).zip(Some(&self.queue))
    }

    pub fn mk_shadowmap(device: &Device, res: u32) -> Texture {
        let format = TextureFormat::Depth32Float;
        let extent = wgpu::Extent3d {
            width: res,
            height: res,
            depth_or_array_layers: N_CASCADES as u32,
        };
        let desc = wgpu::TextureDescriptor {
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            label: Some("shadow map texture"),
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&Texture::depth_compare_sampler());
        Texture {
            texture,
            view,
            sampler,
            format,
            extent,
            transparent: false,
        }
    }

    pub fn set_texture(&mut self, path: impl Into<PathBuf>, tex: Texture) {
        let p = path.into();
        self.texture_cache_paths.insert(p, Arc::new(tex));
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
        if let Some(tex) = self.texture_cache_paths.get(&p) {
            return Some(tex.clone());
        }

        let tex = Arc::new(
            TextureBuilder::try_from_path(&p)?
                .with_label(label)
                .with_mipmaps(self.mipmap_module())
                .build(&self.device, &self.queue),
        );
        self.texture_cache_paths.insert(p, tex.clone());
        Some(tex)
    }

    pub fn read_texture(&self, path: impl Into<PathBuf>) -> Option<&Arc<Texture>> {
        self.texture_cache_paths.get(&path.into())
    }

    pub fn palette(&self) -> Arc<Texture> {
        self.texture_cache_paths
            .get(&*PathBuf::from("assets/sprites/palette.png"))
            .expect("palette not loaded")
            .clone()
    }

    pub fn palette_ref(&self) -> &Texture {
        self.texture_cache_paths
            .get(&*PathBuf::from("assets/sprites/palette.png"))
            .expect("palette not loaded")
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        let present_mode = if vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        if self.sc_desc.present_mode != present_mode {
            self.sc_desc.present_mode = present_mode;
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

    pub fn start_frame(&mut self, sco: &SurfaceTexture) -> (Encoders, TextureView) {
        let end = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("End encoder"),
            });

        for (uni, mat) in self
            .sun_projection
            .iter_mut()
            .zip(self.render_params.value().sun_shadow_proj)
        {
            *uni.value_mut() = mat;
            uni.upload_to_gpu(&self.queue);
        }

        self.projection.upload_to_gpu(&self.queue);
        self.render_params.upload_to_gpu(&self.queue);
        self.lamplights.apply_changes(&self.queue);

        (
            Encoders {
                pbr: None,
                smap: None,
                depth_prepass: None,
                end,
            },
            sco.texture.create_view(&TextureViewDescriptor::default()),
        )
    }

    pub fn get_module(&self, name: &str) -> CompiledModule {
        let p = &mut *self.pipelines.try_borrow_mut().unwrap();

        Pipelines::get_module(
            &mut p.shader_cache,
            &mut p.shader_watcher,
            &self.device,
            name,
        )
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

        {
            profiling::scope!("pbr prepass");
            let mut pbr_enc = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("init encoder"),
                });
            self.pbr.update(self, &mut pbr_enc);
            encs.pbr = Some(pbr_enc.finish());
        }
        {
            profiling::scope!("depth prepass");
            let mut prepass = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("depth prepass encoder"),
                });
            let mut depth_prepass = prepass.begin_render_pass(&RenderPassDescriptor {
                label: Some("depth prepass"),
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
            *enc_dep_ext = Some(prepass.finish());
        }
        if self.render_params.value().shadow_mapping_resolution != 0 {
            profiling::scope!("shadow pass");
            let mut smap_enc = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("shadow map encoder"),
                });
            for (i, u) in self.sun_projection.iter().enumerate() {
                let sun_view = self
                    .sun_shadowmap
                    .texture
                    .create_view(&TextureViewDescriptor {
                        label: Some("sun shadow view"),
                        format: Some(self.sun_shadowmap.format),
                        dimension: Some(TextureViewDimension::D2),
                        aspect: TextureAspect::DepthOnly,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: i as u32,
                        array_layer_count: Some(1),
                    });
                let mut sun_shadow_pass = smap_enc.begin_render_pass(&RenderPassDescriptor {
                    label: Some("sun shadow pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &sun_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    }),
                });

                for obj in objsref.iter() {
                    obj.draw_depth(self, &mut sun_shadow_pass, true, &u.bindgroup);
                }
            }
            *enc_smap_ext = Some(smap_enc.finish());
        }

        if self.render_params.value().ssao_enabled != 0 {
            profiling::scope!("ssao");
            let pipeline = self.get_pipeline(SSAOPipeline);
            let bg = self
                .fbos
                .depth
                .bindgroup(&self.device, &pipeline.get_bind_group_layout(0));

            let mut ssao_pass = encs.end.begin_render_pass(&RenderPassDescriptor {
                label: Some("ssao pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
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
            let mut render_pass = encs.end.begin_render_pass(&RenderPassDescriptor {
                label: Some("main render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
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
                    depth_ops: None,
                    stencil_ops: None,
                }),
            });

            for obj in objsref.iter() {
                obj.draw(self, &mut render_pass);
            }
        }

        render_background(self, encs, &frame);
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
                .chain(encoder.pbr)
                .chain(encoder.smap)
                .chain(Some(encoder.end.finish())),
        );
        if self.tick % 30 == 0 {
            #[cfg(debug_assertions)]
            self.pipelines
                .try_borrow_mut()
                .unwrap()
                .check_shader_updates(&self.device);
        }
        self.tick += 1;
    }

    pub fn create_textures(device: &Device, desc: &SurfaceConfiguration, samples: u32) -> FBOs {
        let size = (desc.width, desc.height);
        let ssao = Texture::create_fbo(
            device,
            size,
            TextureFormat::R8Unorm,
            TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            None,
        );
        let depth = Texture::create_depth_texture(device, size, samples);
        let depth_bg = depth.bindgroup(
            device,
            &Texture::bindgroup_layout(
                device,
                [if samples > 1 {
                    TL::NonfilterableFloatMultisampled
                } else {
                    TL::NonfilterableFloat
                }],
            ),
        );
        FBOs {
            depth,
            depth_bg,
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
                &self.pbr.diffuse_irradiance_cube,
                &self.pbr.specular_prefilter_cube,
                &self.pbr.split_sum_brdf_lut,
                &self.lamplights.lightdata,
                &self.lamplights.lightdata2,
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
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts: layouts,
                    push_constant_ranges: &[],
                });

        let color_states = [Some(wgpu::ColorTargetState {
            format: self.sc_desc.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: vert_shader,
                entry_point: "vert",
                buffers: vertex_buffers,
            },
            fragment: Some(FragmentState {
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
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
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
        frag_shader: Option<&CompiledModule>,
        shadow_map: bool,
    ) -> RenderPipeline {
        self.depth_pipeline_bglayout(
            vertex_buffers,
            vert_shader,
            frag_shader,
            shadow_map,
            &[&self.projection.layout],
        )
    }

    pub fn depth_pipeline_bglayout(
        &self,
        vertex_buffers: &[VertexBufferLayout<'_>],
        vert_shader: &CompiledModule,
        frag_shader: Option<&CompiledModule>,
        shadow_map: bool,
        layouts: &[&BindGroupLayout],
    ) -> RenderPipeline {
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("depth pipeline"),
                    bind_group_layouts: layouts,
                    push_constant_ranges: &[],
                });

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: Some("depth pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: vert_shader,
                entry_point: "vert",
                buffers: vertex_buffers,
            },
            fragment: frag_shader.map(|frag_shader| FragmentState {
                module: frag_shader,
                entry_point: "frag",
                targets: &[],
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(Face::Back),
                front_face: FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
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

    pub fn get_pipeline(&self, obj: impl PipelineBuilder) -> &'static RenderPipeline {
        let pipelines = &mut *self.pipelines.try_borrow_mut().unwrap();
        pipelines.get_pipeline(self, obj, &self.device)
    }

    pub fn mipmap_module(&self) -> CompiledModule {
        // Safety: added at startup
        unsafe {
            self.pipelines
                .try_borrow()
                .unwrap()
                .shader_cache
                .get("mipmap")
                .unwrap_unchecked()
                .clone()
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

#[derive(Copy, Clone, Hash)]
struct SSAOPipeline;

impl PipelineBuilder for SSAOPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str) -> CompiledModule,
    ) -> RenderPipeline {
        let render_pipeline_layout = gfx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("ssao pipeline"),
                bind_group_layouts: &[
                    &Texture::bindgroup_layout(
                        &gfx.device,
                        [if gfx.samples > 1 {
                            TL::NonfilterableFloatMultisampled
                        } else {
                            TL::NonfilterableFloat
                        }],
                    ),
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

        let ssao = mk_module("ssao");

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &ssao,
                entry_point: "vert",
                buffers: &[UvVertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &ssao,
                entry_point: "frag",
                targets: &color_states,
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
        };

        gfx.device.create_render_pipeline(&render_pipeline_desc)
    }
}

#[derive(Hash)]
struct BackgroundPipeline;

impl BackgroundPipeline {
    pub fn bglayout_texs(gfx: &GfxContext) -> BindGroupLayout {
        Texture::bindgroup_layout(&gfx.device, [TL::Float, TL::Cube])
    }
}

impl PipelineBuilder for BackgroundPipeline {
    fn build(
        &self,
        gfx: &GfxContext,
        mut mk_module: impl FnMut(&str) -> CompiledModule,
    ) -> RenderPipeline {
        let bg = &mk_module("background");

        let render_pipeline_layout = gfx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("background"),
                bind_group_layouts: &[
                    &Uniform::<RenderParams>::bindgroup_layout(&gfx.device),
                    &Texture::bindgroup_layout(&gfx.device, [TL::Float]),
                    &Self::bglayout_texs(gfx),
                ],
                push_constant_ranges: &[],
            });

        let color_states = [Some(wgpu::ColorTargetState {
            format: gfx.sc_desc.format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::COLOR,
        })];

        let render_pipeline_desc = RenderPipelineDescriptor {
            label: Some("background pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: bg,
                entry_point: "vert",
                buffers: &[UvVertex::desc()],
            },
            fragment: Some(FragmentState {
                module: bg,
                entry_point: "frag",
                targets: &color_states,
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: gfx.samples,
                ..Default::default()
            },
            multiview: None,
        };
        gfx.device.create_render_pipeline(&render_pipeline_desc)
    }
}

fn render_background(gfx: &GfxContext, encs: &mut Encoders, frame: &&TextureView) {
    profiling::scope!("bg pass");
    let mut bg_pass = encs.end.begin_render_pass(&RenderPassDescriptor {
        label: Some("bg pass"),
        color_attachments: &[Some(RenderPassColorAttachment {
            view: &gfx.fbos.color_msaa,
            resolve_target: Some(frame),
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: true,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: &gfx.fbos.depth.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: false,
            }),
            stencil_ops: None,
        }),
    });

    bg_pass.set_pipeline(gfx.get_pipeline(BackgroundPipeline));
    bg_pass.set_bind_group(0, &gfx.render_params.bindgroup, &[]);
    bg_pass.set_bind_group(1, &gfx.bnoise_bg, &[]);
    bg_pass.set_bind_group(2, &gfx.sky_bg, &[]);
    bg_pass.set_vertex_buffer(0, gfx.screen_uv_vertices.slice(..));
    bg_pass.set_index_buffer(gfx.rect_indices.slice(..), IndexFormat::Uint32);
    bg_pass.draw_indexed(0..6, 0, 0..1);
}
