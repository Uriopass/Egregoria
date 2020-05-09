use crate::engine::GfxContext;
use wgpu::{BindGroupLayout, RenderPass};

mod mesh;
mod spritebatch;
mod textured_mesh;

pub use mesh::*;
pub use spritebatch::*;
pub use textured_mesh::*;

pub type IndexType = u32;

pub struct PreparedPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bindgroupslayouts: Vec<BindGroupLayout>,
}

pub trait HasPipeline: 'static {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline;
}

pub trait Drawable {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>);
}
