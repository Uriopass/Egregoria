#![allow(dead_code)]

use crate::engine::GfxContext;
use image::GenericImageView;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
use wgpu::{CommandEncoderDescriptor, TextureComponentType};

#[derive(Clone)]
pub struct Texture {
    pub width: f32,
    pub height: f32,
    pub texture: Rc<wgpu::Texture>,
    pub view: Rc<wgpu::TextureView>,
    pub sampler: Rc<wgpu::Sampler>,
}

impl Texture {
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
        Self::from_image(ctx, &img, label)
    }

    pub fn from_image(
        ctx: &GfxContext,
        img: &image::DynamicImage,
        label: Option<&'static str>,
    ) -> Option<Self> {
        let rgba = img.as_rgba8().unwrap();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        let buffer = ctx
            .device
            .create_buffer_with_data(&rgba, wgpu::BufferUsage::COPY_SRC);

        let mut encoder = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Texture creation encoder"),
            });

        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &buffer,
                offset: 0,
                bytes_per_row: 4 * dimensions.0,
                rows_per_image: dimensions.1,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            size,
        );

        let cmd_buffer = encoder.finish();

        ctx.queue.submit(&[cmd_buffer]);

        let view = texture.create_default_view();
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        Some(Self {
            texture: Rc::new(texture),
            view: Rc::new(view),
            sampler: Rc::new(sampler),
            width: dimensions.0 as f32,
            height: dimensions.1 as f32,
        })
    }

    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        samples: u32,
    ) -> Self {
        let desc = wgpu::TextureDescriptor {
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            array_layer_count: 1,
            sample_count: samples,
            dimension: wgpu::TextureDimension::D2,
            label: Some("depth texture"),
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_default_view();
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Always,
        });

        Self {
            width: sc_desc.width as f32,
            height: sc_desc.height as f32,
            texture: Rc::new(texture),
            view: Rc::new(view),
            sampler: Rc::new(sampler),
        }
    }

    pub fn bindgroup_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: TextureComponentType::Uint,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
            ],
            label: Some("Texture bindgroup layout"),
        })
    }
}
