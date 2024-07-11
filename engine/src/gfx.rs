use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use wgpu::util::{backend_bits_from_env, BufferInitDescriptor, DeviceExt};
use wgpu::{
    Adapter, Backends, BindGroupLayout, CommandBuffer, CommandEncoder, CommandEncoderDescriptor,
    CompositeAlphaMode, DepthBiasState, Device, Extent3d, Face, FilterMode, FragmentState,
    FrontFace, ImageCopyTexture, ImageDataLayout, InstanceDescriptor, MultisampleState,
    PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, SamplerDescriptor, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, VertexBufferLayout, VertexState,
};
use winit::window::{Fullscreen, Window};

use common::FastMap;
use geom::{vec2, Camera, InfiniteFrustrum, LinearColor, Matrix4, Plane, Vec2, Vec3};

use crate::framework::State;
use crate::meshload::{load_mesh, LoadMeshError};
use crate::passes::{BackgroundPipeline, Pbr};
use crate::perf_counters::PerfCounters;
use crate::{
    bg_layout_litmesh, passes, CompiledModule, Drawable, IndexType, LampLights, Material,
    MaterialID, MaterialMap, Mesh, MetallicRoughness, MipmapGenerator, PipelineKey, Pipelines,
    Texture, TextureBuildError, TextureBuilder, Uniform, UvVertex, WaterPipeline, TL,
};

pub struct FBOs {
    pub(crate) depth: Texture,
    pub(crate) depth_bg: wgpu::BindGroup,
    pub(crate) color_msaa: TextureView,
    pub(crate) ssao: Texture,
    pub(crate) fog: Texture,
    pub(crate) ui_blur: Texture,
    pub format: TextureFormat,
}

pub struct GfxContext {
    pub window: Arc<Window>,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
    pub fbos: FBOs,
    pub mipmap_gen: MipmapGenerator,
    pub size: (u32, u32, f64),
    pub(crate) sc_desc: SurfaceConfiguration,
    pub update_sc: bool,
    settings: Option<GfxSettings>,

    pub(crate) materials: MaterialMap,
    pub(crate) default_material: Material,
    pub tess_material: MaterialID,
    pub tick: u64,
    pub(crate) pipelines: RwLock<Pipelines>,
    pub frustrum: InfiniteFrustrum,
    pub(crate) sun_params: [Uniform<RenderParams>; N_CASCADES],
    pub render_params: Uniform<RenderParams>,
    pub(crate) texture_cache_paths: FastMap<PathBuf, Arc<Texture>>,
    pub(crate) texture_cache_bytes: Mutex<HashMap<u64, Arc<Texture>, common::TransparentHasherU64>>,
    pub null_texture: Texture,
    pub(crate) linear_sampler: wgpu::Sampler,

    pub(crate) mesh_cache: FastMap<PathBuf, Arc<Mesh>>,
    pub(crate) mesh_errors: FastMap<PathBuf, LoadMeshError>,

    pub(crate) samples: u32,
    pub(crate) screen_uv_vertices: wgpu::Buffer,
    pub(crate) rect_indices: wgpu::Buffer,
    pub sun_shadowmap: Texture,
    pub pbr: Pbr,
    pub lamplights: LampLights,
    pub(crate) defines: FastMap<String, String>,
    pub(crate) defines_changed: bool,

    pub simplelit_bg: wgpu::BindGroup,
    pub bnoise_bg: wgpu::BindGroup,
    pub sky_bg: wgpu::BindGroup,
    pub water_bg: wgpu::BindGroup,

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
    Ultra,
}

impl AsRef<str> for ShadowQuality {
    fn as_ref(&self) -> &str {
        match self {
            ShadowQuality::NoShadows => "No Shadows",
            ShadowQuality::Low => "Low",
            ShadowQuality::Medium => "Medium",
            ShadowQuality::High => "High",
            ShadowQuality::Ultra => "Ultra",
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
            4 => ShadowQuality::Ultra,
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
            ShadowQuality::Ultra => Some(4096),
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
    pub fog_shader_debug: bool,
    pub parallel_render: bool,
    pub msaa: bool,
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
            fog_shader_debug: false,
            parallel_render: false,
            msaa: false,
        }
    }
}

pub struct Encoders {
    pub pbr: Option<CommandBuffer>,
    pub smap: Vec<CommandBuffer>,
    pub depth_prepass: Option<CommandBuffer>,
    pub main: Option<CommandBuffer>,
    pub before_main: CommandEncoder,
    pub after_main: CommandEncoder,
    pub gui: Option<CommandBuffer>,
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
    pub gfx: &'a GfxContext,
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
    pub async fn new(window: Arc<Window>) -> Self {
        let mut backends = backend_bits_from_env().unwrap_or_else(Backends::all);
        if std::env::var("RENDERDOC").is_ok() {
            backends = Backends::VULKAN;
        }

        let flags = if cfg!(debug_assertions) {
            // TODO: re enable validation when https://github.com/gfx-rs/wgpu/issues/5231 is fixed
            wgpu::InstanceFlags::DEBUG
        } else {
            wgpu::InstanceFlags::empty()
        };

        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            dx12_shader_compiler: Default::default(),
            flags,
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        let surface = instance.create_surface(window.clone()).unwrap();
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
                    required_features: wgpu::Features::empty(),
                    required_limits: limit,
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
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            format,
            width: win_width,
            height: win_height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        //        let samples = if cfg!(target_arch = "wasm32") { 1 } else { 4 };
        let samples = 1;
        let fbos = Self::create_textures(&device, &sc_desc, samples);
        surface.configure(&device, &sc_desc);

        let screen_uv_vertices = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("screen quad vertices"),
            contents: bytemuck::cast_slice(SCREEN_UV_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let rect_indices = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("screen quad indices"),
            contents: bytemuck::cast_slice(UV_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mipmap_gen = MipmapGenerator::new(&device);

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

        let pbr = Pbr::new(&device, &queue);
        let null_texture = TextureBuilder::empty(1, 1, 1, TextureFormat::Rgba8Unorm)
            .with_srgb(false)
            .with_label("null texture")
            .build_no_queue(&device);

        queue.write_texture(
            ImageCopyTexture {
                texture: &null_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            &[255, 255, 255, 255],
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let linear_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("basic linear sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Linear,
            ..Default::default()
        });

        let mut materials = MaterialMap::default();

        let tess_material = Material::new_raw(
            &device,
            &null_texture,
            MetallicRoughness {
                metallic: 0.0,
                roughness: 1.0,
                tex: None,
            },
            None,
            &null_texture,
        );
        let tess_material_id = materials.insert(tess_material);

        let mut me = Self {
            window,
            size: (win_width, win_height, win_scale_factor),
            sc_desc,
            update_sc: false,
            adapter,
            fbos,
            surface,
            pipelines: RwLock::new(Pipelines::new()),
            materials,
            tess_material: tess_material_id,
            default_material: Material::new_default(&device, &queue, &null_texture),
            tick: 0,
            frustrum: InfiniteFrustrum::new([Plane::X; 5]),
            sun_params: [(); 4].map(|_| Uniform::new(Default::default(), &device)),
            render_params: Uniform::new(Default::default(), &device),
            texture_cache_paths: textures,
            texture_cache_bytes: Default::default(),
            null_texture,
            linear_sampler,
            mesh_cache: Default::default(),
            mesh_errors: Default::default(),
            samples,
            screen_uv_vertices,
            rect_indices,
            simplelit_bg: Uniform::new([0.0f32; 4], &device).bg, // bogus
            sky_bg: Uniform::new([0.0f32; 4], &device).bg,       // bogus
            water_bg: Uniform::new([0.0f32; 4], &device).bg,     // bogus
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
            mipmap_gen,
        };

        me.update_simplelit_bg();

        let palette = TextureBuilder::from_path("assets/sprites/palette.png")
            .with_label("palette")
            .with_sampler(Texture::nearest_sampler())
            .with_mipmaps(&me.mipmap_gen)
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
                .with_mipmaps(&self.mipmap_gen)
                .build(&self.device, &self.queue),
        );
        self.texture_cache_paths.insert(p, tex.clone());
        Ok(tex)
    }

    pub fn read_texture(&self, path: impl Into<PathBuf>) -> Option<&Arc<Texture>> {
        self.texture_cache_paths.get(&path.into())
    }

    pub fn mesh(&mut self, path: &Path) -> Result<Arc<Mesh>, LoadMeshError> {
        if let Some(m) = self.mesh_cache.get(path) {
            return Ok(m.clone());
        }
        if let Some(e) = self.mesh_errors.get(path) {
            return Err(e.clone());
        }
        match load_mesh(self, path) {
            Ok(m) => {
                let m = Arc::new(m);
                self.mesh_cache.insert(path.to_path_buf(), m.clone());
                Ok(m)
            }
            Err(e) => {
                self.mesh_errors.insert(path.to_path_buf(), e.clone());
                Err(e)
            }
        }
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

        let samples = match settings.msaa {
            true => 4,
            false => 1,
        };

        self.set_define_flag("FOG", settings.fog);
        self.set_define_flag("SSAO", settings.ssao);
        self.set_define_flag("TERRAIN_GRID", settings.terrain_grid);
        self.set_define_flag("DEBUG", settings.shader_debug);
        self.set_define_flag("FOG_DEBUG", settings.fog_shader_debug);
        self.set_define_flag("PBR_ENABLED", settings.pbr_enabled);
        self.set_define_flag("MSAA", settings.msaa);

        if self.samples != samples {
            self.samples = samples;
            self.pipelines.write().unwrap().invalidate_all();
            self.fbos = Self::create_textures(&self.device, &self.sc_desc, samples);
            self.update_simplelit_bg();
        }

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
        let mut before_main = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Before main encoder"),
            });
        let after_main = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("After main encoder"),
            });

        for (uni, mat) in self
            .sun_params
            .iter_mut()
            .zip(self.render_params.value().sun_shadow_proj)
        {
            let mut cpy = *self.render_params.value();
            cpy.proj = mat;
            *uni.value_mut() = cpy;
            uni.upload_to_gpu(&self.queue);
        }

        self.render_params.upload_to_gpu(&self.queue);
        self.lamplights
            .apply_changes(&self.queue, &self.device, &mut before_main);

        (
            Encoders {
                pbr: None,
                smap: Default::default(),
                depth_prepass: None,
                main: None,
                before_main,
                after_main,
                gui: None,
            },
            sco.texture.create_view(&TextureViewDescriptor::default()),
        )
    }

    pub fn get_module(&self, name: &str) -> CompiledModule {
        let p = &mut *self.pipelines.write().unwrap();

        Pipelines::get_module(
            &mut p.shader_cache,
            &mut p.shader_watcher,
            &self.device,
            name,
            &self.defines,
            vec![],
        )
    }

    pub fn render<S: State>(
        &mut self,
        encs: &mut Encoders,
        frame: &TextureView,
        state: &mut S,
        render_gui: impl FnOnce(&mut S, GuiRenderContext<'_>),
    ) -> (f32, f32) {
        profiling::scope!("gfx::render_objs");
        self.perf.clear();

        let mut objs = vec![];
        let mut fc = FrameContext {
            objs: &mut objs,
            gfx: self,
        };

        state.render(&mut fc);

        let start_time = Instant::now();

        let objsref = &*objs;

        let mut gui_elapsed = 0.0;

        if self.settings.map(|v| v.parallel_render).unwrap_or(false) {
            rayon::in_place_scope(|scope| {
                scope.spawn(|_| {
                    encs.pbr = self.pbr_prepass();
                });
                scope.spawn(|_| {
                    encs.depth_prepass = Some(self.depth_prepass(objsref));
                });
                scope.spawn(|_| {
                    use rayon::prelude::*;
                    if self.render_params.value().shadow_mapping_resolution != 0 {
                        encs.smap = self.shadow_map_pass(objsref).par_bridge().collect();
                    }
                });
                scope.spawn(|_| {
                    passes::render_ssao(self, &mut encs.before_main);
                    passes::render_fog(self, &mut encs.before_main);

                    passes::render_background(self, &mut encs.after_main, frame);
                    passes::gen_ui_blur(self, &mut encs.after_main, frame);
                });

                scope.spawn(|_| {
                    encs.main = Some(self.main_render_pass(frame, objsref));
                });

                (gui_elapsed, encs.gui) = self.render_gui(frame, state, render_gui);
            });
        } else {
            encs.pbr = self.pbr_prepass();
            encs.depth_prepass = Some(self.depth_prepass(objsref));
            if self.render_params.value().shadow_mapping_resolution != 0 {
                encs.smap = self.shadow_map_pass(objsref).collect();
            }
            passes::render_ssao(self, &mut encs.before_main);
            passes::render_fog(self, &mut encs.before_main);
            encs.main = Some(self.main_render_pass(frame, objsref));
            passes::render_background(self, &mut encs.after_main, frame);
            passes::gen_ui_blur(self, &mut encs.after_main, frame);
            (gui_elapsed, encs.gui) = self.render_gui(frame, state, render_gui);
        }

        (start_time.elapsed().as_secs_f32(), gui_elapsed)
    }

    fn render_gui<S>(
        &self,
        frame: &TextureView,
        state: &mut S,
        render_gui: impl FnOnce(&mut S, GuiRenderContext) + Sized,
    ) -> (f32, Option<CommandBuffer>) {
        profiling::scope!("gfx::render_gui");
        let gui_start = Instant::now();
        let mut gui_enc = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("GUI encoder"),
            });
        render_gui(
            state,
            GuiRenderContext {
                size: self.size,
                gfx: self,
                encoder: &mut gui_enc,
                view: frame,
            },
        );
        (gui_start.elapsed().as_secs_f32(), Some(gui_enc.finish()))
    }

    fn main_render_pass(
        &self,
        frame: &TextureView,
        objsref: &[Box<dyn Drawable>],
    ) -> CommandBuffer {
        profiling::scope!("main render pass");
        let mut main_enc = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("main pass encoder"),
            });

        let ops = wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            store: wgpu::StoreOp::Store,
        };

        let attachment = if self.samples > 1 {
            RenderPassColorAttachment {
                view: &self.fbos.color_msaa,
                resolve_target: Some(frame),
                ops,
            }
        } else {
            RenderPassColorAttachment {
                view: frame,
                resolve_target: None,
                ops,
            }
        };

        let mut render_pass = main_enc.begin_render_pass(&RenderPassDescriptor {
            label: Some("main render pass"),
            color_attachments: &[Some(attachment)],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.fbos.depth.view,
                depth_ops: None,
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_bind_group(0, &self.render_params.bg, &[]);

        for obj in objsref.iter() {
            obj.draw(self, &mut render_pass);
        }

        drop(render_pass);

        main_enc.finish()
    }

    fn pbr_prepass(&self) -> Option<CommandBuffer> {
        if !self.defines.contains_key("PBR_ENABLED") {
            return None;
        }
        profiling::scope!("pbr prepass");
        let mut pbr_enc = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("init encoder"),
            });
        self.pbr.update(self, &mut pbr_enc);
        Some(pbr_enc.finish())
    }

    fn shadow_map_pass<'a>(
        &'a self,
        objsref: &'a [Box<dyn Drawable>],
    ) -> impl Iterator<Item = CommandBuffer> + 'a {
        self.sun_params.iter().enumerate().map(move |(i, u)| {
            profiling::scope!(&format!("cascade shadow pass {}", i));
            let mut smap_enc = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("shadow map encoder"),
                });
            let sun_view = self.sun_shadowmap.layer_view(i as u32);
            self.shadow_map_one_pass(u, objsref, &sun_view, &mut smap_enc);
            smap_enc.finish()
        })
    }

    fn shadow_map_one_pass<'a>(
        &'a self,
        u: &Uniform<RenderParams>,
        objsref: &[Box<dyn Drawable>],
        shadowmap_view: &'a TextureView,
        enc: &'a mut CommandEncoder,
    ) {
        profiling::scope!("cascade shadow pass");
        let mut sun_shadow_pass = enc.begin_render_pass(&RenderPassDescriptor {
            label: Some("sun shadow pass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: shadowmap_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        sun_shadow_pass.set_bind_group(0, &u.bg, &[]);

        for obj in objsref.iter() {
            obj.draw_depth(self, &mut sun_shadow_pass, Some(&u.value().proj));
        }
    }

    fn depth_prepass(&self, objsref: &[Box<dyn Drawable>]) -> CommandBuffer {
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

        depth_prepass.set_bind_group(0, &self.render_params.bg, &[]);

        for obj in objsref.iter() {
            obj.draw_depth(self, &mut depth_prepass, None);
        }
        drop(depth_prepass);
        prepass.finish()
    }

    pub fn finish_frame(&mut self, encoder: Encoders) {
        self.queue.submit(
            encoder
                .depth_prepass
                .into_iter()
                .chain(encoder.pbr)
                .chain(encoder.smap)
                .chain(Some(encoder.before_main.finish()))
                .chain(encoder.main)
                .chain(Some(encoder.after_main.finish()))
                .chain(encoder.gui),
        );
        if self.defines_changed {
            self.defines_changed = false;
            self.pipelines.write().unwrap().invalidate_all();
        }
        if self.tick % 30 == 0 {
            #[cfg(debug_assertions)]
            self.pipelines
                .write()
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
        let ui_blur = passes::gen_blur_texture(device, desc);

        FBOs {
            depth,
            depth_bg,
            color_msaa: if samples > 1 {
                Texture::create_color_msaa(device, desc, samples)
            } else {
                ssao.mip_view(0) // bogus
            },
            ssao,
            fog,
            ui_blur,
            format: desc.format,
        }
    }

    pub fn resize(&mut self, size: (u32, u32, f64)) {
        self.size = size;
        self.sc_desc.width = self.size.0;
        self.sc_desc.height = self.size.1;

        self.surface.configure(&self.device, &self.sc_desc);
        self.fbos = Self::create_textures(&self.device, &self.sc_desc, self.samples);
        self.update_simplelit_bg();
    }

    pub fn update_simplelit_bg(&mut self) {
        self.simplelit_bg = Texture::multi_bindgroup(
            &[
                &self
                    .read_texture("assets/sprites/blue_noise_512.png")
                    .expect("blue noise not initialized"),
                &self.sun_shadowmap,
                &self.pbr.diffuse_irradiance_cube,
                &self.pbr.specular_prefilter_cube,
                &self.pbr.split_sum_brdf_lut,
                &self.fbos.ssao,
                &self.fbos.fog,
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
            &self
                .get_pipeline(BackgroundPipeline)
                .get_bind_group_layout(2),
        );
        self.water_bg = Texture::multi_bindgroup(
            &[&self.fbos.fog],
            &self.device,
            &self.get_pipeline(WaterPipeline).get_bind_group_layout(3),
        );
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
                compilation_options: Default::default(),
                buffers: vertex_buffers,
            },
            fragment: frag_shader.map(|frag_shader| FragmentState {
                module: frag_shader,
                entry_point: "frag",
                compilation_options: Default::default(),
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

    pub fn get_pipeline(&self, obj: impl PipelineKey) -> &'static RenderPipeline {
        let pipelines = &mut *self.pipelines.write().unwrap();
        pipelines.get_pipeline(self, obj, &self.device)
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
