use crate::{Drawable, GfxContext, SpriteBatch, SpriteBatchBuilder, Texture};
use std::iter::FromIterator;
use std::sync::Arc;
use wgpu::RenderPass;

pub struct MultiSpriteBatch {
    sbs: Vec<SpriteBatch>,
}

impl Drawable for MultiSpriteBatch {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        for v in &self.sbs {
            v.draw(gfx, rp);
        }
    }

    fn draw_depth<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        for v in &self.sbs {
            v.draw_depth(gfx, rp);
        }
    }
}

impl FromIterator<SpriteBatch> for MultiSpriteBatch {
    fn from_iter<T: IntoIterator<Item = SpriteBatch>>(iter: T) -> Self {
        Self {
            sbs: iter.into_iter().collect(),
        }
    }
}
