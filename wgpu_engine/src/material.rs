use crate::{GfxContext, Texture, TextureBuilder, ToU8Slice};
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
    pub mat_params: wgpu::Buffer,
    pub metallic_roughness_tex: Option<Arc<Texture>>,
    pub transparent: bool,
}

pub struct MetallicRoughness {
    pub metallic: f32,
    pub roughness: f32,
    pub tex: Option<Arc<Texture>>,
}

const HAS_METALLIC_ROUGHNESS_TEXTURE: u32 = 1 << 0;

#[derive(Copy, Clone)]
#[repr(C)]
struct MaterialParams {
    flags: u32,
    metallic: f32,
    roughness: f32,
}

u8slice_impl!(MaterialParams);

impl Material {
    pub fn new(
        gfx: &GfxContext,
        albedo: Arc<Texture>,
        metallic_roughness: MetallicRoughness,
    ) -> Self {
        Self::new_raw(&gfx.device, albedo, metallic_roughness, gfx.palette_ref())
    }

    pub fn new_raw(
        device: &Device,
        albedo: Arc<Texture>,
        metallic_roughness: MetallicRoughness,
        bogus_tex: &Texture,
    ) -> Self {
        let mut flags = 0;
        if metallic_roughness.tex.is_some() {
            flags |= HAS_METALLIC_ROUGHNESS_TEXTURE;
        }

        let mat_params = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("metallic"),
            contents: ToU8Slice::cast_slice(std::slice::from_ref(&MaterialParams {
                roughness: metallic_roughness.roughness,
                metallic: metallic_roughness.metallic,
                flags,
            })),
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
                    buffer: &mat_params,
                    offset: 0,
                    size: None,
                }),
            },
        ];

        if let Some(ref metallic_roughness_tex) = metallic_roughness.tex {
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&metallic_roughness_tex.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&metallic_roughness_tex.sampler),
            });
        } else {
            // used as placeholder
            entries.push(wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&bogus_tex.view),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&bogus_tex.sampler),
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
            mat_params,
            metallic_roughness_tex: metallic_roughness.tex,
            albedo,
            transparent: false,
        }
    }

    pub(crate) fn bindgroup_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("material layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: Default::default(),
                        min_binding_size: Some(
                            BufferSize::new(std::mem::size_of::<MaterialParams>() as u64).unwrap(),
                        ),
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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
        let bogus = Arc::new(
            TextureBuilder::empty(1, 1, 1, wgpu::TextureFormat::Rgba8Unorm).build(device, queue),
        );

        Self::new_raw(
            device,
            albedo,
            MetallicRoughness {
                roughness: 0.5,
                metallic: 0.0,
                tex: None,
            },
            &bogus,
        )
    }
}
