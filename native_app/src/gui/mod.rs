use std::collections::HashMap;

use imgui::TextureId;
use legion::{system, Entity};
use serde::{Deserialize, Serialize};

use egregoria::engine_interaction::{KeyCode, KeyboardInfo};
pub use follow::FollowEntity;
pub use inspect::*;
use roadbuild::RoadBuildResource;
pub use topgui::*;
use wgpu_engine::GfxContext;

mod bulldozer;
mod follow;
mod inspect;
mod inspected_aura;
mod lotbrush;
mod roadbuild;
mod roadeditor;
mod selectable;
mod specialbuilding;
mod topgui;

pub mod windows;

register_resource_noserialize!(InspectedEntity);
#[derive(Copy, Clone, Default, Debug)]
pub struct InspectedEntity {
    pub e: Option<Entity>,
    pub dirty: bool, // Modified by inspection
    pub dist2: f32,
}

register_system!(hand_reset);
#[system]
pub fn hand_reset(#[resource] info: &KeyboardInfo, #[resource] tool: &mut Tool) {
    if info.just_pressed.contains(&KeyCode::Escape) {
        *tool = Tool::Hand;
    }
}

register_resource!(Tool, "tool");
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Tool {
    Hand,
    RoadbuildStraight,
    RoadbuildCurved,
    RoadEditor,
    Bulldozer,
    LotBrush,
    SpecialBuilding,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum UiTex {
    Road,
    Curved,
    RoadEdit,
    Bulldozer,
    Buildings,
    LotBrush,
}

const UI_TEXTURES: &[(UiTex, &str)] = &[
    (UiTex::Road, "assets/ui/road.png"),
    (UiTex::Curved, "assets/ui/curved.png"),
    (UiTex::RoadEdit, "assets/ui/road_edit.png"),
    (UiTex::Bulldozer, "assets/ui/bulldozer.png"),
    (UiTex::Buildings, "assets/ui/buildings.png"),
    (UiTex::LotBrush, "assets/ui/lotbrush.png"),
];

pub struct UiTextures {
    textures: HashMap<UiTex, TextureId>,
}

impl UiTextures {
    pub fn new(gfx: &GfxContext, renderer: &mut imgui_wgpu::Renderer) -> Self {
        let mut textures = HashMap::with_capacity(UI_TEXTURES.len());
        for &(name, path) in UI_TEXTURES {
            let (img, width, height) = wgpu_engine::Texture::read_image(path)
                .expect(&*format!("Couldn't load gui texture {}", path));

            let mut config = imgui_wgpu::TextureConfig::default();
            config.size.width = width;
            config.size.height = height;

            let imgui_tex = imgui_wgpu::Texture::new(&gfx.device, renderer, config);
            imgui_tex.write(&gfx.queue, &img, width, height);

            textures.insert(name, renderer.textures.insert(imgui_tex));
        }
        Self { textures }
    }

    pub fn get(&self, name: UiTex) -> TextureId {
        *self.textures.get(&name).unwrap()
    }
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Hand
    }
}
