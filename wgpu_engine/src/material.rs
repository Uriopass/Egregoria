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
    pub metallic_v: wgpu::Buffer,
    pub roughness_v: wgpu::Buffer,
    pub metallic_roughness_tex: Option<Arc<Texture>>,
}

pub enum MetallicRoughness {
    Static { metallic: f32, roughness: f32 },
    Texture(Arc<Texture>),
}

impl MetallicRoughness {
    pub fn metallic_value(&self) -> f32 {
        match self {
            MetallicRoughness::Static { metallic, .. } => *metallic,
            MetallicRoughness::Texture(_) => -1.0,
        }
    }

    pub fn roughness_value(&self) -> f32 {
        match self {
            MetallicRoughness::Static { roughness, .. } => *roughness,
            MetallicRoughness::Texture(_) => -1.0,
        }
    }

    pub fn as_texture(&self) -> Option<&Arc<Texture>> {
        match self {
            MetallicRoughness::Static { .. } => None,
            MetallicRoughness::Texture(tex) => Some(tex),
        }
    }

    pub fn into_texture(self) -> Option<Arc<Texture>> {
        match self {
            MetallicRoughness::Static { .. } => None,
            MetallicRoughness::Texture(tex) => Some(tex),
        }
    }
}

impl Material {
    pub fn new(
        gfx: &GfxContext,
        albedo: Arc<Texture>,
        metallic_roughness: MetallicRoughness,
    ) -> Self {
        Self::new_raw(&gfx.device, albedo, metallic_roughness)
    }

    pub fn new_raw(
        device: &Device,
        albedo: Arc<Texture>,
        metallic_roughness: MetallicRoughness,
    ) -> Self {
        let metallic_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("metallic"),
            contents: &metallic_roughness.metallic_value().to_le_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let roughness_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("roughness"),
            contents: &metallic_roughness.roughness_value().to_le_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = &Self::bindgroup_layout(device);

        let mut entries = vec![
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
                    buffer: &metallic_buf,
                    offset: 0,
                    size: None,
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &roughness_buf,
                    offset: 0,
                    size: None,
                }),
            },
        ];

        if let Some(metallic_roughness_tex) = metallic_roughness.as_texture() {
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&metallic_roughness_tex.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&metallic_roughness_tex.sampler),
            });
        } else {
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::TextureView(&albedo.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&albedo.sampler),
            });
        }

        let bgdesc = BindGroupDescriptor {
            layout,
            entries: &entries,
            label: None,
        };
        let bg = device.create_bind_group(&bgdesc);

        Self {
            bg,
            metallic_v: metallic_buf,
            roughness_v: roughness_buf,
            metallic_roughness_tex: metallic_roughness.into_texture(),
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
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
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

        Self::new_raw(
            device,
            albedo,
            MetallicRoughness::Static {
                roughness: 0.5,
                metallic: 0.0,
            },
        )
    }
}
