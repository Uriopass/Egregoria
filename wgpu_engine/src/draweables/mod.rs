use crate::{CompiledShader, GfxContext};
use wgpu::RenderPass;

mod blit_linear;
mod mesh;
mod multispritebatch;
mod shaded_batch;
mod shaded_quad;
mod spritebatch;
mod textured_mesh;

pub use blit_linear::*;
pub use mesh::*;
pub use multispritebatch::*;
pub use shaded_batch::*;
pub use shaded_quad::*;
pub use spritebatch::*;
pub use textured_mesh::*;

pub trait Shaders: 'static {
    fn vert_shader() -> CompiledShader;
    fn frag_shader() -> CompiledShader;
}

pub type IndexType = u32;

pub struct PreparedPipeline(pub wgpu::RenderPipeline);

pub trait Drawable {
    fn create_pipeline(gfx: &GfxContext) -> PreparedPipeline
    where
        Self: Sized;

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>);
}
