use crate::{CompiledShader, GfxContext};
use std::rc::Rc;
use wgpu::{RenderPass, RenderPipeline};

mod blit_linear;
mod lit_mesh;
mod mesh;
mod multispritebatch;
mod shaded_batch;
mod shaded_quad;
mod spritebatch;
mod textured_mesh;

pub use blit_linear::*;
pub use lit_mesh::*;
pub use mesh::*;
pub use multispritebatch::*;
pub use shaded_batch::*;
pub use shaded_quad::*;
pub use spritebatch::*;
pub use textured_mesh::*;

pub trait Shaders: 'static {
    fn vert_shader(device: &wgpu::Device) -> CompiledShader;
    fn frag_shader(device: &wgpu::Device) -> CompiledShader;
}

pub type IndexType = u32;

pub trait Drawable {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline
    where
        Self: Sized;

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>);
}

impl<T: Drawable> Drawable for Rc<T> {
    fn create_pipeline(_: &GfxContext) -> RenderPipeline {
        panic!("dont create pipeline for Rc<T>, create it for T!")
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let s: &T = &*self;
        s.draw(gfx, rp);
    }
}
