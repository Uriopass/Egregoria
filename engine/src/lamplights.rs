use crate::{Texture, TextureBuilder};
use geom::Vec3;
use ordered_float::OrderedFloat;
use wgpu::TextureFormat;

#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
struct EncodedLight(u32);

u8slice_impl!(EncodedLight);

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

struct LightChunkUpdate {
    x: u16,
    y: u16,
    lights: [EncodedLight; 4],
}

pub struct LampLights {
    pub(crate) lightdata: Texture,
    pub(crate) lightdata2: Texture,
    pending_changes: Vec<LightChunkUpdate>,
    pending_changes2: Vec<LightChunkUpdate>,
}

impl LampLights {
    pub const LIGHTCHUNK_SIZE: u32 = 32; // in meters, side length of a light chunk, can contain at most 4 lights
    pub const MAP_SIZE: u32 = 50 * 1024; // in meters, side length of the map
    pub const LIGHTMAP_SIZE: u32 = Self::MAP_SIZE / Self::LIGHTCHUNK_SIZE; // in light chunks

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let lightdata = TextureBuilder::empty(
            Self::LIGHTMAP_SIZE,
            Self::LIGHTMAP_SIZE,
            1,
            TextureFormat::Rgba32Uint,
        )
        .with_label("lightdata")
        .with_sampler(Texture::nearest_sampler())
        .with_srgb(false)
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
        .build(device, queue);

        Self {
            lightdata,
            lightdata2,
            pending_changes: Vec::new(),
            pending_changes2: vec![],
        }
    }

    pub fn reset(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
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

        let mut l = lights.collect::<smallvec::SmallVec<[Vec3; 4]>>();
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
            x: chunk.0,
            y: chunk.1,
            lights: encoded_lights,
        });
        if extra_lights[0].0 != 0 {
            self.pending_changes2.push(LightChunkUpdate {
                x: chunk.0,
                y: chunk.1,
                lights: extra_lights,
            });
        }
    }

    pub fn apply_changes(&mut self, queue: &wgpu::Queue) {
        for change in self.pending_changes.drain(..) {
            // SAFETY: repr(transparent)
            let data: [u32; 4] = unsafe { std::mem::transmute(change.lights) };
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.lightdata.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: change.x as u32,
                        y: change.y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&data),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * 4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }

        for change in self.pending_changes2.drain(..) {
            // SAFETY: repr(transparent)
            let data: [u32; 4] = unsafe { std::mem::transmute(change.lights) };
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.lightdata2.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: change.x as u32,
                        y: change.y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&data),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * 4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}
