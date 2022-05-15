use common::FastMap;
use hecs::Entity;

use imgui::TextureId;
use serde::{Deserialize, Serialize};

use crate::input::{KeyCode, KeyboardInfo};
use crate::uiworld::UiWorld;
use egregoria::Egregoria;
use roadbuild::RoadBuildResource;
use wgpu_engine::GfxContext;

pub mod bulldozer;
pub mod follow;
pub mod inspect;
pub mod inspected_aura;
pub mod lotbrush;
pub mod roadbuild;
pub mod roadeditor;
pub mod selectable;
pub mod specialbuilding;
pub mod topgui;
pub mod trainstation;

pub mod addtrain;
pub mod inputmap;
pub mod windows;

pub use follow::FollowEntity;
pub use inspect::*;
pub use topgui::*;

#[profiling::function]
pub fn run_ui_systems(goria: &Egregoria, uiworld: &mut UiWorld) {
    bulldozer::bulldozer(goria, uiworld);
    inspected_aura::inspected_aura(goria, uiworld);
    lotbrush::lotbrush(goria, uiworld);
    roadbuild::roadbuild(goria, uiworld);
    roadeditor::roadeditor(goria, uiworld);
    selectable::selectable(goria, uiworld);
    specialbuilding::specialbuilding(goria, uiworld);
    trainstation::trainstation(goria, uiworld);
    addtrain::addtrain(goria, uiworld);
    hand_reset(uiworld);
}

#[derive(Copy, Clone, Debug)]
pub struct InspectedEntity {
    pub e: Option<Entity>,
    pub dist2: f32,
}

impl Default for InspectedEntity {
    fn default() -> Self {
        Self {
            e: None,
            dist2: 0.0,
        }
    }
}

#[profiling::function]
pub fn hand_reset(uiworld: &mut UiWorld) {
    let info = uiworld.read::<KeyboardInfo>();
    if info.just_pressed.contains(&KeyCode::Escape) {
        *uiworld.write::<Tool>() = Tool::Hand;
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum Tool {
    Hand,
    RoadbuildStraight,
    RoadbuildCurved,
    RoadEditor,
    Bulldozer,
    LotBrush,
    SpecialBuilding,
    TrainStation,
    AddTrain,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum UiTex {
    Road,
    Curved,
    RoadEdit,
    Bulldozer,
    Buildings,
    LotBrush,
    TrainStation,
    AddTrain,
}

const UI_TEXTURES: &[(UiTex, &str)] = &[
    (UiTex::Road, "assets/ui/road.png"),
    (UiTex::Curved, "assets/ui/curved.png"),
    (UiTex::RoadEdit, "assets/ui/road_edit.png"),
    (UiTex::Bulldozer, "assets/ui/bulldozer.png"),
    (UiTex::Buildings, "assets/ui/buildings.png"),
    (UiTex::LotBrush, "assets/ui/lotbrush.png"),
    (UiTex::TrainStation, "assets/ui/trainstation.png"),
    (UiTex::AddTrain, "assets/ui/traintool.png"),
];

pub struct UiTextures {
    textures: FastMap<UiTex, TextureId>,
}

impl UiTextures {
    pub fn new(gfx: &GfxContext, renderer: &mut imgui_wgpu::Renderer) -> Self {
        let mut textures = common::fastmap_with_capacity(UI_TEXTURES.len());
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
