use crate::GfxContext;
use wgpu::RenderPass;

pub mod heightmap;
mod instanced_mesh;
mod lit_mesh;
mod multispritebatch;
mod spritebatch;
mod water;

pub use instanced_mesh::*;
pub use lit_mesh::*;
pub use multispritebatch::*;
pub use spritebatch::*;
pub use water::*;

use geom::Matrix4;
use std::sync::Arc;

pub type IndexType = u32;

pub trait Drawable: Send + Sync {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>);

    #[allow(unused)]
    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
    }
}

impl<T: ?Sized + Drawable> Drawable for Arc<T> {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let s: &T = self;
        s.draw(gfx, rp);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        let s: &T = self;
        s.draw_depth(gfx, rp, shadow_cascade);
    }
}

impl<T: Drawable> Drawable for Option<T> {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        if let Some(s) = self {
            s.draw(gfx, rp);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        if let Some(s) = self {
            s.draw_depth(gfx, rp, shadow_cascade);
        }
    }
}

impl<T: Drawable> Drawable for [T] {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        for s in self {
            s.draw(gfx, rp);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        for s in self {
            s.draw_depth(gfx, rp, shadow_cascade);
        }
    }
}

impl<T: Drawable> Drawable for Vec<T> {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        for s in self {
            s.draw(gfx, rp);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        for s in self {
            s.draw_depth(gfx, rp, shadow_cascade);
        }
    }
}

impl<T: Drawable, U: Drawable> Drawable for (T, U) {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        self.0.draw(gfx, rp);
        self.1.draw(gfx, rp);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        self.0.draw_depth(gfx, rp, shadow_cascade);
        self.1.draw_depth(gfx, rp, shadow_cascade);
    }
}
