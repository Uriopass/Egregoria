use crate::{GfxContext, Texture, TextureBuilder};
use image::DynamicImage;
use slotmap::new_key_type;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BufferBinding, BufferSize, Device, Queue, SamplerBindingType,
    TextureSampleType,
};

new_key_type! {
    pub struct MaterialID;
}

pub type MaterialMap = slotmap::SlotMap<MaterialID, Material>;

pub struct Material {
    pub bg: BindGroup,
    pub albedo: Arc<Texture>,
    pub metallic: wgpu::Buffer,
    pub roughness: wgpu::Buffer,
}

impl Material {
    pub fn new(gfx: &GfxContext, albedo: Arc<Texture>) -> Self {
        Self::new_raw(&gfx.device, albedo, 0.0, 1.0)
    }

    pub fn new_raw(device: &Device, albedo: Arc<Texture>, metallic: f32, roughness: f32) -> Self {
        let metallic = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("metallic"),
            contents: &metallic.to_le_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let roughness = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("roughness"),
            contents: &roughness.to_le_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = &Self::bindgroup_layout(device);
        let bg = device.create_bind_group(&BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&albedo.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &metallic,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(BufferBinding {
                        buffer: &roughness,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            label: None,
        });

        Self {
            bg,
            metallic,
            roughness,
            albedo,
        }
    }

    pub(crate) fn bindgroup_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("material layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: Default::default(),
                        min_binding_size: Some(BufferSize::new(4).unwrap()),
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: Default::default(),
                        min_binding_size: Some(BufferSize::new(4).unwrap()),
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
            ],
        })
    }

    pub fn new_default(device: &Device, queue: &Queue) -> Self {
        let albedo = Arc::new(
            TextureBuilder::from_img(DynamicImage::ImageRgba8(image::RgbaImage::new(1, 1)))
                .build(device, queue),
        );

        Self::new_raw(device, albedo, 0.0, 0.5)
    }
}
