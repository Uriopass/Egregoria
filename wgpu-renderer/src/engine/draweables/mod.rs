use crate::engine::{FrameContext, GfxContext};
use wgpu::BindGroupLayout;

mod clear_screen;
mod mesh;
mod spritebatch;
mod textured_mesh;

pub use clear_screen::*;
pub use mesh::*;
pub use spritebatch::*;
pub use textured_mesh::*;

pub type IndexType = u32;

pub struct PreparedPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bindgroupslayouts: Vec<BindGroupLayout>,
}

pub trait Drawable: 'static {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline;
    fn draw(&self, ctx: &mut FrameContext);
}
