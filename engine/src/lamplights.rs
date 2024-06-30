use crate::pbuffer::PBuffer;
use crate::{compile_shader, Texture, TextureBuilder};
use common::FastMap;
use geom::Vec3;
use ordered_float::OrderedFloat;
use wgpu::{
    BufferUsages, CommandEncoder, ComputePassDescriptor, Device, Queue, TextureFormat,
    TextureUsages,
};

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct EncodedLight(u32);

impl EncodedLight {
    fn encode(chunk_origin: Vec3, light: Vec3) -> Self {
        let diff = light - chunk_origin;
        let chunk_space = (diff + Vec3::splat(LampLights::LIGHTCHUNK_SIZE as f32))
            / (3 * LampLights::LIGHTCHUNK_SIZE) as f32;
        let x = ((chunk_space.x * (1 << 12) as f32) as u32).min((1 << 12) - 1);
        let y = ((chunk_space.y * (1 << 12) as f32) as u32).min((1 << 12) - 1);
        let z = ((chunk_space.z * (1 << 8) as f32) as u32).min((1 << 8) - 1);
        let v = (x << 20) | (y << 8) | z;
        if v == 0 {
            return Self(1);
        }
        Self(v)
    }
}

pub type LightChunkID = (u16, u16);

#[derive(Copy, Clone)]
#[repr(C)]
struct LightChunkUpdate {
    lights: [EncodedLight; 4],
    lights2: [EncodedLight; 4],
    x: u32,
    y: u32,
    _pad: (u32, u32), // lights (vec4) is 16 bytes aligned
}

u8slice_impl!(LightChunkUpdate);

pub struct LampLights {
    pub(crate) lightdata: Texture,
    pub(crate) lightdata2: Texture,
    pending_changes: Vec<LightChunkUpdate>,
    changes_buffer: PBuffer,
    buffer_layout: wgpu::BindGroupLayout,
    texture_write_bg: wgpu::BindGroup,
    texture_write_pipeline: wgpu::ComputePipeline,
}

impl LampLights {
    pub const LIGHTCHUNK_SIZE: u32 = 32; // in meters, side length of a light chunk, can contain at most 4 lights
    pub const MAP_SIZE: u32 = 50 * 512; // in meters, side length of the map
    pub const LIGHTMAP_SIZE: u32 = Self::MAP_SIZE / Self::LIGHTCHUNK_SIZE; // in light chunks

    pub fn new(device: &Device, queue: &Queue) -> Self {
        let lightdata = TextureBuilder::empty(
            Self::LIGHTMAP_SIZE,
            Self::LIGHTMAP_SIZE,
            1,
            TextureFormat::Rgba32Uint,
        )
        .with_label("lightdata")
        .with_sampler(Texture::nearest_sampler())
        .with_srgb(false)
        .with_usage(TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING)
        .build(device, queue);

        let lightdata2 = TextureBuilder::empty(
            Self::LIGHTMAP_SIZE,
            Self::LIGHTMAP_SIZE,
            1,
            TextureFormat::Rgba32Uint,
        )
        .with_label("lightdata2")
        .with_sampler(Texture::nearest_sampler())
        .with_srgb(false)
        .with_usage(TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING)
        .build(device, queue);

        let texture_write_module =
            compile_shader(device, "compute/texture_write", &FastMap::default());

        let textures_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture_write"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let buffer_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("buffer_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                },
                count: None,
            }],
        });

        let texture_write_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("texture_write"),
            bind_group_layouts: &[&textures_layout, &buffer_layout],
            push_constant_ranges: &[],
        });

        let texture_write_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("texture_write"),
                layout: Some(&texture_write_layout),
                module: &texture_write_module,
                entry_point: "main",
                compilation_options: Default::default(),
            });

        let texture_write_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_write"),
            layout: &textures_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&lightdata.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&lightdata2.view),
                },
            ],
        });

        Self {
            lightdata,
            lightdata2,
            pending_changes: Vec::new(),
            changes_buffer: PBuffer::new(BufferUsages::COPY_DST | BufferUsages::STORAGE),
            buffer_layout,
            texture_write_pipeline,
            texture_write_bg,
        }
    }

    pub fn reset(&mut self, device: &Device, queue: &Queue) {
        *self = Self::new(device, queue);
    }

    pub fn chunk_id(pos: Vec3) -> LightChunkID {
        let x = pos.x / Self::LIGHTCHUNK_SIZE as f32;
        let y = pos.y / Self::LIGHTCHUNK_SIZE as f32;
        let xu = if x < Self::LIGHTMAP_SIZE as f32 && x >= 0.0 {
            x as u16
        } else {
            Self::LIGHTMAP_SIZE as u16 - 1
        };
        let yu = if y < Self::LIGHTMAP_SIZE as f32 && y >= 0.0 {
            y as u16
        } else {
            Self::LIGHTMAP_SIZE as u16 - 1
        };
        (xu, yu)
    }

    pub fn register_update(&mut self, chunk: LightChunkID, lights: impl Iterator<Item = Vec3>) {
        let origin = Vec3::new(
            chunk.0 as f32 * Self::LIGHTCHUNK_SIZE as f32,
            chunk.1 as f32 * Self::LIGHTCHUNK_SIZE as f32,
            0.0,
        );

        let mut l = lights.collect::<Vec<Vec3>>();
        l.sort_unstable_by_key(|x| {
            OrderedFloat(x.distance2(origin + Vec3::splat(Self::LIGHTCHUNK_SIZE as f32 / 2.0)))
        });

        let mut encoded_lights = [EncodedLight(0); 4];
        let mut extra_lights = [EncodedLight(0); 4];
        for (i, light) in l.into_iter().enumerate() {
            if i < 4 {
                encoded_lights[i] = EncodedLight::encode(origin, light);
            } else if i < 8 {
                extra_lights[i - 4] = EncodedLight::encode(origin, light);
            } else {
                break;
            }
        }
        self.pending_changes.push(LightChunkUpdate {
            x: chunk.0 as u32,
            y: chunk.1 as u32,
            lights: encoded_lights,
            lights2: extra_lights,
            _pad: (0, 0),
        });
    }

    pub fn apply_changes(&mut self, queue: &Queue, device: &Device, encoder: &mut CommandEncoder) {
        if self.pending_changes.is_empty() {
            return;
        }

        self.changes_buffer
            .write_qd(queue, device, bytemuck::cast_slice(&self.pending_changes));

        let Some(buffer_bg) = self.changes_buffer.bindgroup(device, &self.buffer_layout) else {
            self.pending_changes.clear();
            return;
        };

        let mut compute = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("lamp lights update"),
            timestamp_writes: None,
        });

        compute.set_pipeline(&self.texture_write_pipeline);
        compute.set_bind_group(0, &self.texture_write_bg, &[]);
        compute.set_bind_group(1, &buffer_bg, &[]);
        compute.dispatch_workgroups((self.pending_changes.len() as u32).div_ceil(64), 1, 1);
        self.pending_changes.clear();
    }
}
