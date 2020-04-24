use crate::engine::context::{Context, FrameContext};
use wgpu::BindGroupLayout;

mod clear_screen;
mod mesh;
mod rainbow;
mod textured_mesh;

pub use clear_screen::*;
pub use mesh::*;
pub use rainbow::*;
pub use textured_mesh::*;

pub type IndexType = u32;

pub struct PreparedPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bindgroupslayouts: Vec<BindGroupLayout>,
}

pub trait Draweable: 'static {
    fn create_pipeline(gfx: &Context) -> PreparedPipeline;
    fn draw(&self, ctx: &mut FrameContext);
}
