#![allow(dead_code)]

use crate::{compile_shader, GfxContext};
use image::GenericImageView;
use std::fs::File;
use std::io::Read;
use std::num::NonZeroU32;
use std::path::Path;
use std::rc::Rc;
use wgpu::{
    BindGroup, BindGroupLayout, CommandEncoderDescriptor, Device, Extent3d,
    PipelineLayoutDescriptor, Sampler, TextureComponentType, TextureCopyView, TextureDataLayout,
    TextureFormat, TextureUsage, TextureViewDescriptor,
};

pub struct OwnedTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub format: TextureFormat,
    pub extent: Extent3d,
}

#[derive(Clone)]
pub struct Texture {
    pub texture: Rc<wgpu::Texture>,
    pub view: Rc<wgpu::TextureView>,
    pub sampler: Rc<wgpu::Sampler>,
    pub format: TextureFormat,
    pub extent: Extent3d,
}

#[derive(Clone)]
pub struct MultisampledTexture {
    pub target: Texture,
    pub multisampled_buffer: Rc<wgpu::TextureView>,
}

impl Texture {
    pub fn read_image(p: impl AsRef<Path>) -> Option<(Vec<u8>, u32, u32)> {
        let mut buf = vec![];
        let mut f = File::open(p).ok()?;
        f.read_to_end(&mut buf).ok()?;
        image::load_from_memory(&*buf).ok().map(|x| {
            let w = x.width();
            let h = x.height();
            (x.into_rgba8().into_raw(), w, h)
        })
    }

    pub fn from_path(
        ctx: &GfxContext,
        p: impl AsRef<Path>,
        label: Option<&'static str>,
    ) -> Option<Self> {
        let mut buf = vec![];
        let mut f = File::open(p).ok()?;
        f.read_to_end(&mut buf).ok()?;
        Texture::from_bytes(&ctx, &buf, label)
    }

    pub fn from_bytes(ctx: &GfxContext, bytes: &[u8], label: Option<&'static str>) -> Option<Self> {
        let img = image::load_from_memory(bytes).ok()?;
        Some(Self::from_image(ctx, &img, label))
    }

    pub fn from_image(
        ctx: &GfxContext,
        img: &image::DynamicImage,
        label: Option<&'static str>,
    ) -> Self {
        let rgba = img
            .as_rgba8()
            .expect("Trying to use non rgha8 image as texture");
        let dimensions = img.dimensions();

        let extent = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let mip_level_count = 6;
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size: extent,
            mip_level_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST | TextureUsage::OUTPUT_ATTACHMENT,
        });

        ctx.queue.write_texture(
            TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * extent.width,
                rows_per_image: extent.height,
            },
            extent,
        );

        generate_mipmaps(&ctx.device, &ctx.queue, &texture, format, mip_level_count);

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = Self::default_sampler(&ctx.device);

        Self {
            texture: Rc::new(texture),
            view: Rc::new(view),
            sampler: Rc::new(sampler),
            format,
            extent,
        }
    }

    pub fn create_fbo(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        format: wgpu::TextureFormat,
        usage: TextureUsage,
        samples: Option<u32>,
    ) -> Texture {
        let extent = wgpu::Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        };
        let desc = wgpu::TextureDescriptor {
            format,
            usage,
            size: extent,
            mip_level_count: 1,
            sample_count: samples.unwrap_or(1),
            dimension: wgpu::TextureDimension::D2,
            label: Some("depth texture"),
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = Self::default_sampler(&device);

        Self {
            texture: Rc::new(texture),
            view: Rc::new(view),
            sampler: Rc::new(sampler),
            format,
            extent,
        }
    }

    pub fn create_depth_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        samples: u32,
    ) -> Self {
        Self::create_fbo(
            device,
            sc_desc,
            TextureFormat::Depth32Float,
            TextureUsage::OUTPUT_ATTACHMENT,
            Some(samples),
        )
    }

    pub fn create_light_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> Self {
        Self::create_fbo(
            device,
            sc_desc,
            TextureFormat::R32Float,
            TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
            None,
        )
    }

    pub fn create_ui_texture(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        Self::create_fbo(
            device,
            sc_desc,
            TextureFormat::Rgba8Unorm,
            TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
            None,
        )
    }

    pub fn create_color_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        samples: u32,
    ) -> MultisampledTexture {
        let target = Self::create_fbo(
            device,
            sc_desc,
            TextureFormat::Rgba8UnormSrgb,
            TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
            None,
        );

        let multisample_desc = &wgpu::TextureDescriptor {
            format: target.format,
            size: Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            label: Some("color texture"),
        };

        MultisampledTexture {
            target,
            multisampled_buffer: Rc::new(
                device
                    .create_texture(multisample_desc)
                    .create_view(&TextureViewDescriptor::default()),
            ),
        }
    }

    pub fn create_normal_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        samples: u32,
    ) -> MultisampledTexture {
        let target = Self::create_fbo(
            device,
            sc_desc,
            TextureFormat::Rgba16Float,
            TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
            None,
        );

        let multisample_desc = &wgpu::TextureDescriptor {
            format: target.format,
            size: Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            usage: TextureUsage::OUTPUT_ATTACHMENT,
            mip_level_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            label: Some("normal texture"),
        };

        MultisampledTexture {
            target,
            multisampled_buffer: Rc::new(
                device
                    .create_texture(multisample_desc)
                    .create_view(&TextureViewDescriptor::default()),
            ),
        }
    }

    pub fn bindgroup_layout(
        device: &wgpu::Device,
        component_type: TextureComponentType,
    ) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
            ],
            label: Some("Texture bindgroup layout"),
        })
    }

    pub fn bindgroup_layout_float(device: &wgpu::Device) -> BindGroupLayout {
        Self::bindgroup_layout(device, TextureComponentType::Float)
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

    fn default_sampler(device: &Device) -> Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
        })
    }

    pub fn srgb_format() -> TextureFormat {
        TextureFormat::Rgba8UnormSrgb
    }

    pub fn into_owned(self) -> Option<OwnedTexture> {
        Some(OwnedTexture {
            texture: Rc::<wgpu::Texture>::try_unwrap(self.texture).ok()?,
            view: std::rc::Rc::<wgpu::TextureView>::try_unwrap(self.view).ok()?,
            sampler: std::rc::Rc::<wgpu::Sampler>::try_unwrap(self.sampler).ok()?,
            format: TextureFormat::R8Unorm,
            extent: Default::default(),
        })
    }
}

fn generate_mipmaps(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    mip_count: u32,
) {
    let vs_module =
        device.create_shader_module(compile_shader("assets/shaders/mipmap.vert", None).0);
    let fs_module =
        device.create_shader_module(compile_shader("assets/shaders/mipmap.frag", None).0);

    let bglayout = Texture::bindgroup_layout(&device, TextureComponentType::Uint);

    let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bglayout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mipmaps"),
        layout: Some(&layout),
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
            cull_mode: wgpu::CullMode::None,
            ..Default::default()
        }),
        primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
        color_states: &[format.into()],
        depth_stencil_state: None,
        vertex_state: wgpu::VertexStateDescriptor {
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
        },
        sample_count: 1,
        sample_mask: !0,
        alpha_to_coverage_enabled: false,
    });

    let bind_group_layout = pipeline.get_bind_group_layout(0);

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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
            texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("mip"),
                format: None,
                dimension: None,
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip,
                level_count: NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: None,
            })
        })
        .collect::<Vec<_>>();

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

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &views[target_mip],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&pipeline);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.draw(0..4, 0..1);
        }
        queue.submit(Some(encoder.finish()));
    }
}
