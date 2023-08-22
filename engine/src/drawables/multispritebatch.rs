use crate::{Drawable, GfxContext, SpriteBatch};
use geom::Matrix4;
use wgpu::RenderPass;

pub struct MultiSpriteBatch {
    sbs: Vec<SpriteBatch>,
}

impl Drawable for MultiSpriteBatch {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        self.sbs.draw(gfx, rp);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_cascade: Option<&Matrix4>,
    ) {
        self.sbs.draw_depth(gfx, rp, shadow_cascade);
    }
}

impl FromIterator<SpriteBatch> for MultiSpriteBatch {
    fn from_iter<T: IntoIterator<Item = SpriteBatch>>(iter: T) -> Self {
        Self {
            sbs: iter.into_iter().collect(),
        }
    }
}
