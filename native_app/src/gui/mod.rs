use common::FastMap;
use std::path::Path;
use std::sync::Arc;

use egui::{ColorImage, ImageData, TextureHandle, TextureId, TextureOptions};
use engine::yakui::YakuiWrapper;
use engine::GfxContext;

pub mod chat;
pub mod follow;
pub mod hud;
pub mod inspect;
pub mod windows;

pub use follow::FollowEntity;
pub use hud::*;

#[derive(Default)]
pub struct UiTextures {
    textures: FastMap<String, TextureHandle>,
    yakui_textures: FastMap<String, yakui::TextureId>,
}

impl UiTextures {
    pub fn new(gfx: &mut GfxContext, yakui: &mut YakuiWrapper, ctx: &mut egui::Context) -> Self {
        let texdirs = [("assets/ui", ""), ("assets/ui/icons", "icon/")];

        let mut textures: FastMap<String, TextureHandle> = Default::default();
        let mut yakui_textures: FastMap<String, yakui::TextureId> = Default::default();

        for (prefix, path) in texdirs.iter().flat_map(|(path, prefix)| {
            common::saveload::walkdir(Path::new(path)).map(move |x| (prefix, x))
        }) {
            let name = prefix.to_string() + path.file_stem().unwrap().to_str().unwrap();

            let (img, width, height) = engine::Texture::read_image(&path)
                .unwrap_or_else(|| panic!("Couldn't load gui texture {:?}", &path));

            let h = ctx.load_texture(
                &name,
                ImageData::Color(Arc::new(ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &img,
                ))),
                TextureOptions::LINEAR,
            );

            textures.insert(name.clone(), h);
            yakui_textures.insert(name, yakui.add_texture(gfx, &path));
        }
        Self {
            textures,
            yakui_textures,
        }
    }

    pub fn get(&self, name: &str) -> TextureId {
        match self.textures.get(name) {
            None => panic!("Couldn't find texture (egui) {}", name),
            Some(x) => x.id(),
        }
    }

    pub fn get_yakui(&self, name: &str) -> yakui::TextureId {
        match self.yakui_textures.get(name) {
            None => panic!("Couldn't find texture (yakui) {}", name),
            Some(x) => *x,
        }
    }

    pub fn try_get(&self, name: &str) -> Option<TextureId> {
        self.textures.get(name).map(TextureHandle::id)
    }
}
