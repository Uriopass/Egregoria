use crate::{Drawable, GfxContext, SpriteBatch, SpriteBatchBuilder, Texture};
use std::iter::FromIterator;
use wgpu::{RenderPass, RenderPipeline};

pub struct MultiSpriteBatchBuilder {
    sbs: Vec<SpriteBatchBuilder>,
}

#[derive(Clone)]
pub struct MultiSpriteBatch {
    sbs: Vec<SpriteBatch>,
}

impl MultiSpriteBatchBuilder {
    pub fn from_paths(ctx: &GfxContext, paths: &[&'static str]) -> Self {
        Self {
            sbs: paths
                .iter()
                .map(move |path| SpriteBatchBuilder::from_path(ctx, path))
                .collect(),
        }
    }

    pub fn new(texs: Vec<Texture>) -> Self {
        Self {
            sbs: texs.into_iter().map(SpriteBatchBuilder::new).collect(),
        }
    }

    pub fn n_texs(&self) -> usize {
        self.sbs.len()
    }

    pub fn build(&self, gfx: &GfxContext) -> MultiSpriteBatch {
        let sb: Vec<SpriteBatch> = self.sbs.iter().filter_map(|sb| sb.build(gfx)).collect();
        MultiSpriteBatch { sbs: sb }
    }

    pub fn clear(&mut self) {
        for sbb in &mut self.sbs {
            sbb.clear();
        }
    }

    pub fn sb(&mut self, id: usize) -> &mut SpriteBatchBuilder {
        &mut self.sbs[id]
    }
}

impl Drawable for MultiSpriteBatch {
    fn create_pipeline(_gfx: &GfxContext) -> RenderPipeline {
        unimplemented!()
    }

    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        for v in &self.sbs {
            v.draw(gfx, rp);
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
