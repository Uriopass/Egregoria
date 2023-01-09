use crate::{Drawable, GfxContext, SpriteBatch};
use wgpu::RenderPass;

pub struct MultiMesh {
    sbs: Vec<SpriteBatch>,
}

impl Drawable for MultiMesh {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        self.sbs.draw(gfx, rp);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        self.sbs.draw_depth(gfx, rp, shadow_map, proj);
    }
}

impl FromIterator<SpriteBatch> for MultiMesh {
    fn from_iter<T: IntoIterator<Item = SpriteBatch>>(iter: T) -> Self {
        Self {
            sbs: iter.into_iter().collect(),
        }
    }
}
