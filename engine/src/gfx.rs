use crate::background::BackgroundPipeline;
use crate::pbr::PBR;
use crate::perf_counters::PerfCounters;
use crate::{
    background, bg_layout_litmesh, fog, ssao, CompiledModule, Drawable, IndexType, LampLights,
    Material, MaterialID, MaterialMap, PipelineBuilder, Pipelines, Texture, TextureBuildError,
    TextureBuilder, Uniform, UvVertex, TL,
};
use common::FastMap;
use geom::{vec2, Camera, InfiniteFrustrum, LinearColor, Matrix4, Plane, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wgpu::util::{backend_bits_from_env, BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, Backends, BindGroupLayout, BlendState, CommandBuffer, CommandEncoder,
    CommandEncoderDescriptor, CompositeAlphaMode, DepthBiasState, Device, Face, FragmentState,
    FrontFace, InstanceDescriptor, MultisampleState, PipelineLayoutDescriptor, PrimitiveState,
    Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Surface, SurfaceConfiguration, SurfaceTexture, TextureAspect,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexBufferLayout, VertexState,
};
use winit::window::{Fullscreen, Window};

pub struct FBOs {
    pub(crate) depth: Texture,
    pub(crate) depth_bg: wgpu::BindGroup,
    pub(crate) color_msaa: TextureView,
    pub(crate) ssao: Texture,
    pub(crate) fog: Texture,
    pub format: TextureFormat,
}

pub struct GfxContext {
    pub window: Window,
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub fbos: FBOs,
    pub size: (u32, u32, f64),
    pub(crate) sc_desc: SurfaceConfiguration,
    pub update_sc: bool,
    settings: Option<GfxSettings>,

    pub(crate) materials: MaterialMap,
    pub(crate) default_material: Material,
    pub tick: u64,
    pub(crate) pipelines: RefCell<Pipelines>,
    pub frustrum: InfiniteFrustrum,
    pub(crate) sun_params: [Uniform<RenderParams>; N_CASCADES],
    pub render_params: Uniform<RenderParams>,
    pub(crate) texture_cache_paths: FastMap<PathBuf, Arc<Texture>>,
    pub(crate) texture_cache_bytes: Mutex<HashMap<u64, Arc<Texture>, common::TransparentHasherU64>>,
    pub(crate) null_texture: Texture,

    pub(crate) samples: u32,
    pub(crate) screen_uv_vertices: wgpu::Buffer,
    pub(crate) rect_indices: wgpu::Buffer,
    pub sun_shadowmap: Texture,
    pub pbr: PBR,
    pub lamplights: LampLights,
    pub(crate) defines: FastMap<String, String>,
    pub(crate) defines_changed: bool,

    pub simplelit_bg: wgpu::BindGroup,
    pub bnoise_bg: wgpu::BindGroup,
    pub sky_bg: wgpu::BindGroup,
    #[allow(dead_code)] // keep adapter alive
    pub(crate) adapter: Adapter,

    pub perf: PerfCounters,
}

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
pub enum ShadowQuality {
    NoShadows,
    Low,
    Medium,
    High,
    TooHigh,
}

impl AsRef<str> for ShadowQuality {
    fn as_ref(&self) -> &str {
        match self {
            ShadowQuality::NoShadows => "No Shadows",
            ShadowQuality::Low => "Low",
            ShadowQuality::Medium => "Medium",
            ShadowQuality::High => "High",
            ShadowQuality::TooHigh => "Too High",
        }
    }
}

impl From<u8> for ShadowQuality {
    fn from(v: u8) -> Self {
        match v {
            0 => ShadowQuality::NoShadows,
            1 => ShadowQuality::Low,
            2 => ShadowQuality::Medium,
            3 => ShadowQuality::High,
            4 => ShadowQuality::TooHigh,
            _ => ShadowQuality::High,
        }
    }
}

impl ShadowQuality {
    pub fn size(&self) -> Option<u32> {
        match self {
            ShadowQuality::Low => Some(512),
            ShadowQuality::Medium => Some(1024),
            ShadowQuality::High => Some(2048),
            ShadowQuality::TooHigh => Some(4096),
            ShadowQuality::NoShadows => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GfxSettings {
    pub vsync: bool,
    pub fullscreen: bool,
    pub shadows: ShadowQuality,
    pub fog: bool,
    pub ssao: bool,
    pub terrain_grid: bool,
    pub shader_debug: bool,
    pub pbr_enabled: bool,
}

impl Default for GfxSettings {
    fn default() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            shadows: ShadowQuality::High,
            fog: true,
            ssao: true,
            terrain_grid: true,
            shader_debug: false,
            pbr_enabled: true,
        }
    }
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
    pub proj: Matrix4,
    pub inv_proj: Matrix4,
    pub sun_shadow_proj: [Matrix4; N_CASCADES],
    pub cam_pos: Vec3,
    pub _pad: f32,
    pub cam_dir: Vec3, // Vec3s need to be 16 aligned
    pub _pad4: f32,
    pub sun: Vec3,
    pub _pad2: f32,
    pub sun_col: LinearColor,
    pub grass_col: LinearColor,
    pub sand_col: LinearColor,
    pub sea_col: LinearColor,
    pub viewport: Vec2,
    pub unproj_pos: Vec2,
    pub time: f32,
    pub time_always: f32,
    pub shadow_mapping_resolution: i32,
    pub terraforming_mode_radius: f32,
}

#[cfg(test)]
#[test]
fn test_renderparam_size() {
    println!(
        "size of RenderParams: {}",
        std::mem::size_of::<RenderParams>()
    );
    assert!(std::mem::size_of::<RenderParams>() < 1024);
}

impl Default for RenderParams {
    fn default() -> Self {
        Self {
            proj: Matrix4::zero(),
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
            unproj_pos: Default::default(),
            time: 0.0,
            time_always: 0.0,
            shadow_mapping_resolution: 2048,
            terraforming_mode_radius: 0.0,
            _pad: 0.0,
            _pad2: 0.0,
            _pad4: 0.0,
        }
    }
}

u8slice_impl!(RenderParams);

pub struct GuiRenderContext<'a> {
    pub gfx: &'a mut GfxContext,
    pub encoder: &'a mut CommandEncoder,
    pub view: &'a TextureView,
    pub size: (u32, u32, f64),
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
    pub async fn new(window: Window) -> Self {
        let mut backends = backend_bits_from_env().unwrap_or_else(Backends::all);
        if std::env::var("RENDERDOC").is_ok() {
            backends = Backends::VULKAN;
        }

        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default() | wgpu::InstanceFlags::DEBUG,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("failed to find a suitable adapter");

        let limit = wgpu::Limits::default();

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
            .expect("failed to find a suitable device");

        let capabilities = surface.get_capabilities(&adapter);

        let format = *capabilities
            .formats
            .iter()
            .find(|x| x.is_srgb())
            .unwrap_or_else(|| &capabilities.formats[0]);

        let win_width = window.inner_size().width;
        let win_height = window.inner_size().height;
        let win_scale_factor = window.scale_factor();

        let sc_desc = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: win_width,
            height: win_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        let samples = if cfg!(target_arch = "wasm32") { 1 } else { 4 };
        let fbos = Self::create_textures(&device, &sc_desc, samples);
        surface.configure(&device, &sc_desc);

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
        let null_texture = TextureBuilder::empty(1, 1, 1, TextureFormat::Rgba8Unorm)
            .with_srgb(false)
            .with_label("null texture")
            .build(&device, &queue);

        let mut me = Self {
            window,
            size: (win_width, win_height, win_scale_factor),
            sc_desc,
            update_sc: false,
            adapter,
            fbos,
            surface,
            pipelines: RefCell::new(Pipelines::new(&device)),
            materials: Default::default(),
            default_material: Material::new_default(&device, &queue, &null_texture),
            tick: 0,
            frustrum: InfiniteFrustrum::new([Plane::X; 5]),
            sun_params: [(); 4].map(|_| Uniform::new(Default::default(), &device)),
            render_params: Uniform::new(Default::default(), &device),
            texture_cache_paths: textures,
            texture_cache_bytes: Default::default(),
            null_texture,
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
            defines: Default::default(),
            defines_changed: false,
            settings: None,
            perf: Default::default(),
        };

        me.update_simplelit_bg();

        let palette = TextureBuilder::from_path("assets/sprites/palette.png")
            .with_label("palette")
            .with_sampler(Texture::nearest_sampler())
            .with_mipmaps(me.mipmap_module())
            .build(&me.device, &me.queue);
        me.set_texture("assets/sprites/palette.png", palette);

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

    pub fn set_define_flag(&mut self, name: &str, inserted: bool) {
        if self.defines.contains_key(name) == inserted {
            return;
        }
        self.defines_changed = true;
        if inserted {
            self.defines.insert(name.to_string(), String::new());
        } else {
            self.defines.remove(name);
        }
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
    ) -> Result<Arc<Texture>, TextureBuildError> {
        self.texture_inner(path.into(), label)
    }

    fn texture_inner(
        &mut self,
        p: PathBuf,
        label: &'static str,
    ) -> Result<Arc<Texture>, TextureBuildError> {
        if let Some(tex) = self.texture_cache_paths.get(&p) {
            return Ok(tex.clone());
        }

        let tex = Arc::new(
            TextureBuilder::try_from_path(&p)?
                .with_label(label)
                .with_mipmaps(self.mipmap_module())
                .build(&self.device, &self.queue),
        );
        self.texture_cache_paths.insert(p, tex.clone());
        Ok(tex)
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

    pub fn update_settings(&mut self, settings: GfxSettings) {
        if self.settings == Some(settings) {
            return;
        }

        if Some(settings.fullscreen) != self.settings.map(|s| s.fullscreen) {
            self.window.set_fullscreen(
                settings
                    .fullscreen
                    .then(|| Fullscreen::Borderless(self.window.current_monitor())),
            )
        }

        let present_mode = if settings.vsync {
            wgpu::PresentMode::AutoVsync
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        if self.sc_desc.present_mode != present_mode {
            self.sc_desc.present_mode = present_mode;
            self.update_sc = true;
        }

        let params = self.render_params.value_mut();
        params.shadow_mapping_resolution = settings.shadows.size().unwrap_or(0) as i32;

        if let Some(v) = settings.shadows.size() {
            if self.sun_shadowmap.extent.width != v {
                self.sun_shadowmap = GfxContext::mk_shadowmap(&self.device, v);
                self.update_simplelit_bg();
            }
        }

        self.set_define_flag("FOG", settings.fog);
        self.set_define_flag("SSAO", settings.ssao);
        self.set_define_flag("TERRAIN_GRID", settings.terrain_grid);
        self.set_define_flag("DEBUG", settings.shader_debug);
        self.set_define_flag("PBR_ENABLED", settings.pbr_enabled);

        self.settings = Some(settings);
    }

    pub fn set_time(&mut self, time: f32) {
        self.render_params.value_mut().time = time;
    }

    pub fn set_camera(&mut self, cam: Camera) {
        self.render_params.value_mut().proj = cam.proj_cache;
        self.render_params.value_mut().inv_proj = cam.inv_proj_cache;

        self.frustrum = InfiniteFrustrum::from_reversez_invviewproj(cam.eye(), cam.inv_proj_cache);
    }

    pub fn start_frame(&mut self, sco: &SurfaceTexture) -> (Encoders, TextureView) {
        let end = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("End encoder"),
            });

        for (uni, mat) in self
            .sun_params
            .iter_mut()
            .zip(self.render_params.value().sun_shadow_proj)
        {
            let mut cpy = self.render_params.value().clone();
            cpy.proj = mat;
            *uni.value_mut() = cpy;
            uni.upload_to_gpu(&self.queue);
        }

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
            &self.defines,
        )
    }

    pub fn render_objs(
        &mut self,
        encs: &mut Encoders,
        frame: &TextureView,
        mut prepare: impl FnMut(&mut FrameContext<'_>),
    ) -> Duration {
        profiling::scope!("gfx::render_objs");
        self.perf.clear();

        let mut objs = vec![];
        let mut fc = FrameContext {
            objs: &mut objs,
            gfx: self,
        };

        prepare(&mut fc);

        let start_time = Instant::now();

        let objsref = &*objs;
        let enc_dep_ext = &mut encs.depth_prepass;
        let enc_smap_ext = &mut encs.smap;

        if self.defines.contains_key("PBR_ENABLED") {
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
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            depth_prepass.set_bind_group(0, &self.render_params.bindgroup, &[]);

            for obj in objsref.iter() {
                obj.draw_depth(self, &mut depth_prepass, None);
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
            for (i, u) in self.sun_params.iter().enumerate() {
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
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                sun_shadow_pass.set_bind_group(0, &u.bindgroup, &[]);

                for obj in objsref.iter() {
                    obj.draw_depth(self, &mut sun_shadow_pass, Some(&u.value().proj));
                }
            }
            *enc_smap_ext = Some(smap_enc.finish());
        }

        if self.defines.contains_key("SSAO") {
            ssao::render_ssao(self, &mut encs.end);
        }

        fog::render_fog(self, &mut encs.end);

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
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.fbos.depth.view,
                    depth_ops: None,
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_bind_group(0, &self.render_params.bindgroup, &[]);

            for obj in objsref.iter() {
                obj.draw(self, &mut render_pass);
            }
        }

        background::render_background(self, encs, &frame);

        start_time.elapsed()
    }

    pub fn render_gui(
        &mut self,
        encoders: &mut Encoders,
        frame: &TextureView,
        mut render_gui: impl FnMut(GuiRenderContext<'_>),
    ) {
        profiling::scope!("gfx::render_gui");
        render_gui(GuiRenderContext {
            size: self.size,
            gfx: self,
            encoder: &mut encoders.end,
            view: frame,
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
        if self.defines_changed {
            self.defines_changed = false;
            self.pipelines
                .try_borrow_mut()
                .unwrap()
                .invalidate_all(&self.defines, &self.device);
        }
        if self.tick % 30 == 0 {
            #[cfg(debug_assertions)]
            self.pipelines
                .try_borrow_mut()
                .unwrap()
                .check_shader_updates(&self.defines, &self.device);
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
        let fog = Texture::create_fbo(
            device,
            (size.0 / 3, size.1 / 3),
            TextureFormat::Rgba16Float,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            None,
        );
        FBOs {
            depth,
            depth_bg,
            color_msaa: Texture::create_color_msaa(device, desc, samples),
            ssao,
            fog,
            format: desc.format,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32, scale_factor: f64) {
        self.size = (width, height, scale_factor);
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
                &self.fbos.fog,
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

        let starfield = self.texture("assets/sprites/starfield.png", "starfield");
        self.sky_bg = Texture::multi_bindgroup(
            &[&*starfield, &self.fbos.fog, &self.pbr.environment_cube],
            &self.device,
            &BackgroundPipeline::bglayout_texs(&self),
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
            &[&self.render_params.layout],
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
        // unwrap safety: added at startup
        self.pipelines
            .try_borrow()
            .unwrap()
            .shader_cache
            .get("mipmap")
            .unwrap()
            .clone()
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
