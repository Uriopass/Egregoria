use common::FastMap;
use engine::yakui::YakuiWrapper;
use engine::GfxContext;
use std::path::Path;

#[derive(Default)]
pub struct UiTextures {
    yakui_textures: FastMap<String, yakui::TextureId>,
}

impl UiTextures {
    pub fn new(gfx: &mut GfxContext, yakui: &mut YakuiWrapper) -> Self {
        let texdirs = [("assets/ui", ""), ("assets/ui/icons", "icon/")];

        let mut yakui_textures: FastMap<String, yakui::TextureId> = Default::default();

        for (prefix, path) in texdirs.iter().flat_map(|(path, prefix)| {
            common::saveload::walkdir(Path::new(path)).map(move |x| (prefix, x))
        }) {
            let name = prefix.to_string() + path.file_stem().unwrap().to_str().unwrap();

            yakui_textures.insert(name, yakui.load_texture(gfx, &path));
        }
        Self { yakui_textures }
    }

    pub fn get(&self, name: &str) -> yakui::TextureId {
        match self.yakui_textures.get(name) {
            None => panic!("Couldn't find texture (yakui) {}", name),
            Some(x) => *x,
        }
    }

    pub fn try_get(&self, name: &str) -> Option<yakui::TextureId> {
        self.yakui_textures.get(name).copied()
    }
}
