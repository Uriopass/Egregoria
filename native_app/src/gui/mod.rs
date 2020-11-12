use crate::gui::lotbrush::LotBrushResource;
use crate::gui::specialbuilding::SpecialBuildingResource;
use crate::gui::windows::debug::DebugObjs;
use common::inspect::InspectedEntity;
use egregoria::engine_interaction::{KeyCode, KeyboardInfo};
use egregoria::Egregoria;
use imgui::TextureId;
use legion::system;
use movable::MovableSystem;
use roadbuild::RoadBuildResource;
use roadeditor::RoadEditorResource;
use std::collections::HashMap;
use wgpu_engine::GfxContext;

mod bulldozer;
mod follow;
mod inspect;
mod inspected_aura;
mod lotbrush;
mod movable;
mod roadbuild;
mod roadeditor;
mod selectable;
mod specialbuilding;
mod topgui;
pub mod windows;

pub use follow::FollowEntity;
pub use inspect::*;
pub use topgui::*;

pub fn setup_gui(goria: &mut Egregoria) {
    goria
        .schedule
        .add_system(selectable::selectable_select_system())
        .add_system(selectable::selectable_cleanup_system())
        .add_system(roadbuild::roadbuild_system())
        .add_system(roadeditor::roadeditor_system())
        .add_system(bulldozer::bulldozer_system())
        .add_system(lotbrush::lotbrush_system())
        .add_system(inspected_aura::inspected_aura_system())
        .add_system(specialbuilding::special_building_system())
        .add_system(hand_reset_system())
        .add_system(movable::movable_system(MovableSystem::default()));

    goria.insert(InspectedEntity::default());
    goria.insert(FollowEntity::default());
    goria.insert(Tool::default());
    goria.insert(DebugObjs::default());

    goria.insert(RoadBuildResource::default());
    goria.insert(RoadEditorResource::default());
    goria.insert(LotBrushResource::default());
    goria.insert(SpecialBuildingResource::default());
}

#[system]
pub fn hand_reset(#[resource] info: &KeyboardInfo, #[resource] tool: &mut Tool) {
    if info.just_pressed.contains(&KeyCode::Escape) {
        *tool = Tool::Hand;
    }
}

#[derive(Copy, Clone)]
pub enum Tool {
    Hand,
    RoadbuildStraight,
    RoadbuildCurved,
    RoadEditor,
    Bulldozer,
    LotBrush,
    SpecialBuilding,
}

const Z_TOOL: f32 = 0.9;

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
