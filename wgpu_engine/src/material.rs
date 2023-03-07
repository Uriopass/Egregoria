use crate::{GfxContext, Texture, TextureBuilder};
use image::DynamicImage;
use slotmap::new_key_type;
use std::sync::Arc;
use wgpu::{BindGroup, Device, Queue};

new_key_type! {
    pub struct MaterialID;
}

pub type MaterialMap = slotmap::SlotMap<MaterialID, Material>;

pub struct Material {
    pub bg: BindGroup,
    pub albedo: Arc<Texture>,
}

impl Material {
    pub fn new(gfx: &GfxContext, albedo: Arc<Texture>) -> Self {
        Self {
            bg: albedo.bindgroup(&gfx.device, &Texture::bindgroup_layout(&gfx.device)),
            albedo,
        }
    }

    pub fn new_default(device: &Device, queue: &Queue) -> Self {
        let albedo = Arc::new(
            TextureBuilder::from_img(DynamicImage::ImageRgba8(image::RgbaImage::new(1, 1)))
                .build(device, queue),
        );
        Self {
            bg: albedo.bindgroup(device, &Texture::bindgroup_layout(device)),
            albedo,
        }
    }
}
