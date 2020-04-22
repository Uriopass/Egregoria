use crate::engine::context::{FrameContext, GfxContext};
use wgpu::{BindGroupLayout, CommandBuffer};

mod clear_screen;
mod mesh;
mod rainbow;

pub use clear_screen::*;
pub use mesh::*;
pub use rainbow::*;

pub type IndexType = u32;

pub struct PreparedPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bindgroupslayouts: Vec<BindGroupLayout>,
}

pub trait Draweable: 'static {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline;
    fn draw(&self, ctx: &mut FrameContext);
}
