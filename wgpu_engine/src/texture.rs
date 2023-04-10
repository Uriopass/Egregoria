#![allow(dead_code)]

use crate::CompiledModule;
use image::{DynamicImage, GenericImageView, Rgba32FImage};
use std::fs::File;
use std::io::Read;
use std::num::{NonZeroU32, NonZeroU8};
use std::path::Path;
use wgpu::{
    BindGroup, BindGroupLayout, BindGroupLayoutEntry, CommandEncoderDescriptor, Device, Extent3d,
    ImageCopyTexture, ImageDataLayout, PipelineLayoutDescriptor, SamplerDescriptor, TextureFormat,
    TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
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

    pub fn bindgroup_layout(device: &Device, it: impl IntoIterator<Item = TL>) -> BindGroupLayout {
        let entries: Vec<BindGroupLayoutEntry> = it
            .into_iter()
            .enumerate()
            .flat_map(|(i, bgtype)| {
                vec![
                    BindGroupLayoutEntry {
                        binding: (i * 2) as u32,
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
                            sample_type: if matches!(
                                bgtype,
                                TL::Depth | TL::DepthMultisampled | TL::DepthArray
                            ) {
                                TextureSampleType::Depth
                            } else {
                                TextureSampleType::Float {
                                    filterable: !matches!(
                                        bgtype,
                                        TL::NonfilterableFloat | TL::NonfilterableFloatMultisampled
                                    ),
                                }
                            },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: (i * 2 + 1) as u32,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Sampler(
                            if matches!(bgtype, TL::Depth | TL::DepthMultisampled | TL::DepthArray)
                            {
                                wgpu::SamplerBindingType::Comparison
                            } else {
                                wgpu::SamplerBindingType::Filtering
                            },
                        ),
                        count: None,
                    },
                ]
                .into_iter()
            })
            .collect::<Vec<_>>();
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

    pub fn multi_bindgroup(
        texs: &[&Texture],
        device: &Device,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let entries = texs
            .iter()
            .enumerate()
            .flat_map(|(i, tex)| {
                vec![
                    wgpu::BindGroupEntry {
                        binding: (i * 2) as u32,
                        resource: wgpu::BindingResource::TextureView(&tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: (i * 2 + 1) as u32,
                        resource: wgpu::BindingResource::Sampler(&tex.sampler),
                    },
                ]
            })
            .collect::<Vec<_>>();
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &entries,
            label: None,
        })
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
}

pub struct TextureBuilder<'a> {
    img: Option<DynamicImage>,
    dimensions: (u32, u32, u32),
    format: Option<TextureFormat>,
    sampler: SamplerDescriptor<'static>,
    label: &'a str,
    srgb: bool,
    mipmaps: Option<CompiledModule>,
    mipmaps_no_gen: bool,
    fixed_mipmaps: Option<u32>,
}

impl<'a> TextureBuilder<'a> {
    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn with_sampler(mut self, sampler: SamplerDescriptor<'static>) -> Self {
        self.sampler = sampler;
        self
    }

    pub fn with_srgb(mut self, srgb: bool) -> Self {
        self.srgb = srgb;
        self
    }

    pub fn with_mipmaps(mut self, mipmaps: CompiledModule) -> Self {
        self.mipmaps = Some(mipmaps);
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

    pub(crate) fn from_path(p: impl AsRef<Path>) -> Self {
        let r = p.as_ref();
        if let Some(x) = Self::try_from_path(r) {
            x
        } else {
            panic!(
                "texture not found at path: {} (in dir: {:?})",
                r.display(),
                std::env::current_dir().as_ref().map(|x| x.display())
            )
        }
    }

    pub(crate) fn try_from_path(p: impl AsRef<Path>) -> Option<Self> {
        let p = p.as_ref();
        let mut buf = vec![];
        let mut f = File::open(p).ok()?;
        f.read_to_end(&mut buf).ok()?;
        Self::from_bytes(&buf)
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Option<Self> {
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
        }

        let img = image::load_from_memory(bytes).ok()?;
        Some(Self::from_img(img))
    }

    pub(crate) fn from_img(img: DynamicImage) -> Self {
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
        }
    }

    pub(crate) fn empty(w: u32, h: u32, d: u32, format: TextureFormat) -> Self {
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
        }
    }

    pub fn build(self, device: &Device, queue: &wgpu::Queue) -> Texture {
        let extent = Extent3d {
            width: self.dimensions.0,
            height: self.dimensions.1,
            depth_or_array_layers: self.dimensions.2,
        };

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

        let mip_level_count =
            if self.mipmaps.is_some() || self.mipmaps_no_gen || self.fixed_mipmaps.is_some() {
                if let Some(v) = self.fixed_mipmaps {
                    v
                } else {
                    let m = self.dimensions.0.min(self.dimensions.1);
                    (m.next_power_of_two().trailing_zeros()).max(1)
                }
            } else {
                1
            };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(self.label),
            size: extent,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
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
                    bytes_per_row: NonZeroU32::new(pixwidth * extent.width),
                    rows_per_image: None,
                },
                extent,
            );

            if mip_level_count > 1 {
                if let Some(module) = self.mipmaps {
                    generate_mipmaps(
                        device,
                        queue,
                        &texture,
                        format,
                        mip_level_count,
                        module,
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
        {
            sampl.anisotropy_clamp = Some(NonZeroU8::new(16).unwrap());
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

fn generate_mipmaps(
    device: &Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    format: TextureFormat,
    mip_count: u32,
    module: CompiledModule,
    label: &str,
) {
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
            module: &module,
            entry_point: "vert",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: "frag",
            targets: &[Some(format.into())],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let bind_group_layout = pipeline.get_bind_group_layout(0);

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

    let views = (0..mip_count)
        .map(|mip| {
            texture.create_view(&TextureViewDescriptor {
                label: Some("mip"),
                format: None,
                dimension: None,
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip,
                mip_level_count: NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: None,
            })
        })
        .collect::<Vec<_>>();

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    for target_mip in 1..mip_count as usize {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&views[target_mip - 1]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("mip generation for {label} - {target_mip}")),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &views[target_mip],
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&pipeline);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.draw(0..4, 0..1);
    }
    queue.submit(Some(encoder.finish()));
}
