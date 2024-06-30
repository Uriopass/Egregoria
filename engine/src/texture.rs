#![allow(dead_code)]

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use derive_more::{Display, From};
use image::{DynamicImage, GenericImageView};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, CommandEncoder,
    CommandEncoderDescriptor, Device, Extent3d, ImageCopyTexture, ImageDataLayout, MapMode,
    PipelineLayoutDescriptor, RenderPipeline, SamplerDescriptor, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use common::FastMap;

use crate::{compile_shader, CompiledModule};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: wgpu::Sampler,
    pub format: TextureFormat,
    pub extent: Extent3d,
    pub transparent: bool,
}

/// TextureLayout
pub enum TL {
    Depth,
    DepthMultisampled,
    DepthArray,
    Float,
    NonfilterableFloat,
    NonfilterableFloatMultisampled,
    Cube,
    UInt,
    SInt,
}

impl Texture {
    pub fn read_image(p: impl AsRef<Path>) -> Option<(Vec<u8>, u32, u32)> {
        let mut buf = vec![];
        let mut f = File::open(p).ok()?;
        f.read_to_end(&mut buf).ok()?;
        image::load_from_memory(&buf).ok().map(|x| {
            let w = x.width();
            let h = x.height();
            (x.into_rgba8().into_raw(), w, h)
        })
    }

    pub fn create_fbo(
        device: &Device,
        (width, height): (u32, u32),
        format: TextureFormat,
        usage: TextureUsages,
        samples: Option<u32>,
    ) -> Texture {
        let extent = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            format,
            usage,
            size: extent,
            mip_level_count: 1,
            sample_count: samples.unwrap_or(1),
            dimension: wgpu::TextureDimension::D2,
            label: Some("fbo texture"),
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&Self::linear_sampler());

        Self {
            texture,
            view,
            sampler,
            format,
            extent,
            transparent: false,
        }
    }

    pub fn create_depth_texture(device: &Device, size: (u32, u32), samples: u32) -> Texture {
        Self::create_fbo(
            device,
            size,
            TextureFormat::Depth32Float,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            Some(samples),
        )
    }

    pub fn create_color_msaa(
        device: &Device,
        sc_desc: &wgpu::SurfaceConfiguration,
        samples: u32,
    ) -> wgpu::TextureView {
        let multisample_desc = &wgpu::TextureDescriptor {
            format: sc_desc.format,
            size: Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth_or_array_layers: 1,
            },
            usage: TextureUsages::RENDER_ATTACHMENT,
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            label: Some("color texture"),
            view_formats: &[],
        };

        device
            .create_texture(multisample_desc)
            .create_view(&TextureViewDescriptor::default())
    }

    pub fn n_mips(&self) -> u32 {
        self.texture.mip_level_count()
    }

    pub fn bindgroup_layout_entries(
        binding_offset: u32,
        it: impl Iterator<Item = TL>,
    ) -> impl Iterator<Item = BindGroupLayoutEntry> {
        it.enumerate().flat_map(move |(i, bgtype)| {
            std::iter::once(BindGroupLayoutEntry {
                binding: binding_offset + (i * 2) as u32,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: matches!(
                        bgtype,
                        TL::NonfilterableFloatMultisampled | TL::DepthMultisampled
                    ),
                    view_dimension: match bgtype {
                        TL::Cube => TextureViewDimension::Cube,
                        TL::DepthArray => TextureViewDimension::D2Array,
                        _ => TextureViewDimension::D2,
                    },
                    sample_type: match bgtype {
                        TL::Depth | TL::DepthMultisampled | TL::DepthArray => {
                            TextureSampleType::Depth
                        }
                        TL::UInt => TextureSampleType::Uint,
                        TL::SInt => TextureSampleType::Sint,
                        _ => TextureSampleType::Float {
                            filterable: !matches!(
                                bgtype,
                                TL::NonfilterableFloat | TL::NonfilterableFloatMultisampled
                            ),
                        },
                    },
                },
                count: None,
            })
            .chain(std::iter::once(BindGroupLayoutEntry {
                binding: binding_offset + (i * 2 + 1) as u32,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Sampler(
                    if matches!(bgtype, TL::Depth | TL::DepthMultisampled | TL::DepthArray) {
                        wgpu::SamplerBindingType::Comparison
                    } else {
                        wgpu::SamplerBindingType::Filtering
                    },
                ),
                count: None,
            }))
        })
    }

    pub fn bindgroup_layout(device: &Device, it: impl IntoIterator<Item = TL>) -> BindGroupLayout {
        let entries: Vec<BindGroupLayoutEntry> =
            Self::bindgroup_layout_entries(0, it.into_iter()).collect();
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &entries,
            label: Some("Texture bindgroup layout"),
        })
    }

    pub fn bindgroup(&self, device: &Device, layout: &BindGroupLayout) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: None,
        })
    }

    pub fn multi_bindgroup_entries<'a>(
        binding_offset: u32,
        texs: &'a [&Texture],
    ) -> impl Iterator<Item = BindGroupEntry<'a>> {
        texs.iter().enumerate().flat_map(move |(i, tex)| {
            std::iter::once(BindGroupEntry {
                binding: binding_offset + (i * 2) as u32,
                resource: wgpu::BindingResource::TextureView(&tex.view),
            })
            .chain(std::iter::once(BindGroupEntry {
                binding: binding_offset + (i * 2 + 1) as u32,
                resource: wgpu::BindingResource::Sampler(&tex.sampler),
            }))
        })
    }

    pub fn multi_bindgroup(
        texs: &[&Texture],
        device: &Device,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let entries = Self::multi_bindgroup_entries(0, texs).collect::<Vec<_>>();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &entries,
            label: None,
        })
    }

    pub fn save_to_file(
        &self,
        device: &Device,
        queue: &wgpu::Queue,
        path: PathBuf,
        mip_level: u32,
    ) {
        match self.format {
            TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => {}
            _ => {
                log::error!("save_to_file not implemented for format {:?}", self.format);
                return;
            }
        }

        let w = self.extent.width >> mip_level;
        let h = self.extent.height >> mip_level;
        let size = w * h;

        debug_assert!(self.extent.depth_or_array_layers == 1);

        let block_size = self.format.block_copy_size(None).unwrap();

        let image_data_buf = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("save_to_file"),
            size: (block_size * size) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        }));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("save_to_file"),
        });

        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &image_data_buf,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(block_size * w),
                    rows_per_image: None,
                },
            },
            self.extent,
        );

        queue.submit(std::iter::once(encoder.finish()));

        let image_data_buf_cpy = image_data_buf.clone();
        image_data_buf.slice(..).map_async(MapMode::Read, move |v| {
            if v.is_err() {
                log::error!("Failed to map buffer for reading for save_to_file");
                return;
            }

            let v = image_data_buf_cpy.slice(..).get_mapped_range();

            let Some(rgba) = image::RgbaImage::from_raw(w, h, v.to_vec()) else {
                log::error!("Failed to create image from buffer for save_to_file");
                return;
            };

            if let Err(e) = rgba.save(path) {
                log::error!("Failed to save image to file: {}", e);
            }
        });
    }

    pub fn depth_compare_sampler() -> SamplerDescriptor<'static> {
        SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            border_color: None,
            ..Default::default()
        }
    }

    pub fn linear_sampler() -> SamplerDescriptor<'static> {
        SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        }
    }

    pub fn nearest_sampler() -> SamplerDescriptor<'static> {
        SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }
    }

    pub fn mip_view(&self, mip_level: u32) -> TextureView {
        self.texture.create_view(&TextureViewDescriptor {
            label: Some("texture mip"),
            format: None,
            dimension: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: mip_level,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: None,
        })
    }

    pub fn layer_view(&self, layer: u32) -> TextureView {
        self.texture.create_view(&TextureViewDescriptor {
            label: Some("texture array one layer view"),
            format: None,
            dimension: Some(TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: layer,
            array_layer_count: Some(1),
        })
    }
}

#[derive(Debug, Display, From)]
pub enum TextureBuildError {
    Io(io::Error),
    Image(image::ImageError),
}

#[derive(Clone)]
pub struct TextureBuilder<'a> {
    img: Option<DynamicImage>,
    dimensions: (u32, u32, u32),
    format: Option<TextureFormat>,
    sampler: SamplerDescriptor<'static>,
    label: &'a str,
    srgb: bool,
    mipmaps: Option<&'a MipmapGenerator>,
    mipmaps_no_gen: bool,
    fixed_mipmaps: Option<u32>,
    usage: TextureUsages,
    no_anisotropy: bool,
    sample_count: u32,
}

impl<'a> TextureBuilder<'a> {
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn with_usage(mut self, usage: TextureUsages) -> Self {
        self.usage = usage;
        self
    }

    pub fn with_sampler(mut self, sampler: SamplerDescriptor<'static>) -> Self {
        self.sampler = sampler;
        self
    }

    pub fn with_no_anisotropy(mut self) -> Self {
        self.no_anisotropy = true;
        self
    }

    pub fn with_srgb(mut self, srgb: bool) -> Self {
        self.srgb = srgb;
        self
    }

    pub fn with_mipmaps(mut self, mipmapgen: &'a MipmapGenerator) -> Self {
        self.mipmaps = Some(mipmapgen);
        self
    }

    pub fn with_mipmaps_no_gen(mut self) -> Self {
        self.mipmaps_no_gen = true;
        self
    }

    pub fn with_fixed_mipmaps(mut self, v: u32) -> Self {
        self.fixed_mipmaps = Some(v);
        self
    }

    pub fn from_path(p: impl AsRef<Path>) -> Self {
        let r = p.as_ref();
        match Self::try_from_path(r) {
            Ok(x) => x,
            Err(e) => {
                panic!(
                    "texture not found at path: {} (in dir: {:?}): {}",
                    r.display(),
                    std::env::current_dir().as_ref().map(|x| x.display()),
                    e,
                )
            }
        }
    }

    pub fn try_from_path(p: impl AsRef<Path>) -> Result<Self, TextureBuildError> {
        let p = p.as_ref();
        let mut buf = vec![];
        let mut f = File::open(p)?;
        f.read_to_end(&mut buf)?;
        Self::from_bytes(&buf)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TextureBuildError> {
        /*
        if bytes.starts_with(b"#?RADIANCE") {
            let irradiance = radiant::load(bytes).ok()?;

            let data_mapped = irradiance
                .data
                .into_iter()
                .map(|pixel| [pixel.r, pixel.g, pixel.b, 1.0])
                .collect::<Vec<_>>();

            let image_irradiance = DynamicImage::ImageRgba32F(Rgba32FImage::from_raw(
                irradiance.width as u32,
                irradiance.height as u32,
                bytemuck::cast_vec(data_mapped),
            )?);
            return Some(Self::from_img(image_irradiance));
        }*/

        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_img(img))
    }

    pub fn from_img(img: DynamicImage) -> Self {
        Self {
            dimensions: (img.dimensions().0, img.dimensions().1, 1),
            img: Some(img),
            format: None,
            sampler: Texture::linear_sampler(),
            label: "texture without label",
            srgb: true,
            mipmaps: None,
            mipmaps_no_gen: false,
            fixed_mipmaps: None,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            no_anisotropy: false,
            sample_count: 1,
        }
    }

    pub fn empty(w: u32, h: u32, d: u32, format: TextureFormat) -> Self {
        Self {
            img: None,
            sampler: Texture::linear_sampler(),
            dimensions: (w, h, d),
            format: Some(format),
            label: "empty texture without label",
            srgb: true,
            mipmaps: None,
            mipmaps_no_gen: false,
            fixed_mipmaps: None,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            no_anisotropy: false,
            sample_count: 1,
        }
    }

    pub fn with_sample_count(mut self, samples: u32) -> Self {
        self.sample_count = samples;
        self
    }

    fn mip_level_count(&self) -> u32 {
        if self.mipmaps.is_some() || self.mipmaps_no_gen || self.fixed_mipmaps.is_some() {
            if let Some(v) = self.fixed_mipmaps {
                v
            } else {
                let m = self.dimensions.0.min(self.dimensions.1);
                (m.next_power_of_two().trailing_zeros()).max(1)
            }
        } else {
            1
        }
    }

    pub fn build_no_queue(self, device: &Device) -> Texture {
        let extent = Extent3d {
            width: self.dimensions.0,
            height: self.dimensions.1,
            depth_or_array_layers: self.dimensions.2,
        };

        let mip_level_count = self.mip_level_count();

        let format = self.format.expect("expected format to be set");

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(self.label),
            size: extent,
            mip_level_count,
            sample_count: self.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: self.usage,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor {
            dimension: Some(if self.dimensions.2 <= 1 {
                TextureViewDimension::D2
            } else {
                TextureViewDimension::Cube
            }),
            ..Default::default()
        });

        let mut sampl = self.sampler.clone();
        if mip_level_count > 1
            && (sampl.min_filter == wgpu::FilterMode::Linear
                || sampl.mag_filter == wgpu::FilterMode::Linear)
            && !self.no_anisotropy
        {
            sampl.anisotropy_clamp = 16;
        }

        let sampler = device.create_sampler(&sampl);

        Texture {
            texture,
            view,
            sampler,
            format: self.format.unwrap(),
            extent,
            transparent: false,
        }
    }

    pub fn build(self, device: &Device, queue: &wgpu::Queue) -> Texture {
        let extent = Extent3d {
            width: self.dimensions.0,
            height: self.dimensions.1,
            depth_or_array_layers: self.dimensions.2,
        };
        let mip_level_count = self.mip_level_count();

        let mut transparent = false;
        let mut format = self.format;
        let mut data = None;

        let img;
        if let Some(img2) = self.img {
            img = match img2 {
                DynamicImage::ImageRgb8(_) => DynamicImage::ImageRgba8(img2.to_rgba8()),
                _ => {
                    for (_, _, pixel) in img2.pixels() {
                        if pixel.0[3] != 255 {
                            transparent = true;
                            break;
                        }
                    }
                    img2
                }
            };

            let (img_format, img_data, pixwidth): (TextureFormat, &[u8], u32) = match img {
                DynamicImage::ImageRgba8(ref img) => (
                    if self.srgb {
                        TextureFormat::Rgba8UnormSrgb
                    } else {
                        TextureFormat::Rgba8Unorm
                    },
                    img,
                    4,
                ),
                DynamicImage::ImageLuma8(ref gray) => (TextureFormat::R8Unorm, gray, 1),
                DynamicImage::ImageRgba32F(ref img) => {
                    (TextureFormat::Rgba32Float, bytemuck::cast_slice(img), 16)
                }
                _ => unimplemented!("unsupported format {:?}", img.color()),
            };
            format = Some(img_format);
            data = Some((img_data, pixwidth));
        }

        let format = format.unwrap();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(self.label),
            size: extent,
            mip_level_count,
            sample_count: self.sample_count,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: self.usage,
            view_formats: &[],
        });

        if let Some((data, pixwidth)) = data {
            queue.write_texture(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: Default::default(),
                },
                data,
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(pixwidth * extent.width),
                    rows_per_image: None,
                },
                extent,
            );

            if mip_level_count > 1 {
                if let Some(mipmapgen) = self.mipmaps {
                    mipmapgen.generate_mipmaps(
                        device,
                        queue,
                        &texture,
                        format,
                        mip_level_count,
                        self.label,
                    );
                }
            }
        }

        let view = texture.create_view(&TextureViewDescriptor {
            dimension: Some(if self.dimensions.2 <= 1 {
                TextureViewDimension::D2
            } else {
                TextureViewDimension::Cube
            }),
            ..Default::default()
        });

        let mut sampl = self.sampler.clone();
        if mip_level_count > 1
            && (sampl.min_filter == wgpu::FilterMode::Linear
                || sampl.mag_filter == wgpu::FilterMode::Linear)
            && !self.no_anisotropy
        {
            sampl.anisotropy_clamp = 16;
        }

        let sampler = device.create_sampler(&sampl);

        Texture {
            texture,
            view,
            sampler,
            format,
            extent,
            transparent,
        }
    }
}

pub struct MipmapGenerator {
    pipelines: RwLock<FastMap<TextureFormat, RenderPipeline>>,
    sampler: wgpu::Sampler,
    module: CompiledModule,
}

impl MipmapGenerator {
    pub fn new(device: &Device) -> Self {
        let module = compile_shader(device, "mipmap", &FastMap::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("mip"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            pipelines: Default::default(),
            sampler,
            module,
        }
    }
}

impl MipmapGenerator {
    pub fn generate_mipmaps(
        &self,
        device: &Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        format: TextureFormat,
        mip_count: u32,
        label: &str,
    ) {
        self.with_pipeline(device, format, |pipe| {
            let views = (0..mip_count)
                .map(|mip| {
                    texture.create_view(&TextureViewDescriptor {
                        label: Some("mip"),
                        format: None,
                        dimension: None,
                        aspect: wgpu::TextureAspect::All,
                        base_mip_level: mip,
                        mip_level_count: Some(1),
                        base_array_layer: 0,
                        array_layer_count: None,
                    })
                })
                .collect::<Vec<_>>();

            let mut encoder =
                device.create_command_encoder(&CommandEncoderDescriptor { label: None });
            for target_mip in 1..mip_count as usize {
                self.mipmap_one(
                    &mut encoder,
                    device,
                    pipe,
                    &views[target_mip - 1],
                    &views[target_mip],
                    label,
                );
            }
            queue.submit(Some(encoder.finish()));
        });
    }

    pub fn with_pipeline(
        &self,
        device: &Device,
        format: TextureFormat,
        f: impl FnOnce(&RenderPipeline),
    ) {
        let b = self.pipelines.read().unwrap();
        if b.contains_key(&format) {
            f(&b[&format]);
            return;
        }
        drop(b);

        let bglayout = Texture::bindgroup_layout(device, [TL::Float]);

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bglayout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&*format!("mipmaps {format:?}")),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &self.module,
                entry_point: "vert",
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &self.module,
                entry_point: "frag",
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        self.pipelines.write().unwrap().insert(format, pipeline);

        f(&self.pipelines.read().unwrap()[&format]);
    }

    pub fn mipmap_one(
        &self,
        encoder: &mut CommandEncoder,
        device: &Device,
        pipeline: &RenderPipeline,
        src: &TextureView,
        dst: &TextureView,
        label: &str,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(src),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: None,
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("mip generation for {label}")),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rpass.set_pipeline(pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..3, 0..1);
    }
}
