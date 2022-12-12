use common::FastMap;
use hecs::Entity;

use egui::{ColorImage, ImageData, TextureHandle, TextureId, TextureOptions};
use serde::{Deserialize, Serialize};

use crate::uiworld::UiWorld;
use egregoria::engine_interaction::WorldCommand;
use egregoria::Egregoria;
use roadbuild::RoadBuildResource;

pub(crate) mod bulldozer;
pub(crate) mod follow;
pub(crate) mod inspect;
pub(crate) mod inspected_aura;
pub(crate) mod lotbrush;
pub(crate) mod roadbuild;
pub(crate) mod roadeditor;
pub(crate) mod selectable;
pub(crate) mod specialbuilding;
pub(crate) mod topgui;

pub(crate) mod addtrain;
pub(crate) mod windows;

pub(crate) use follow::FollowEntity;
pub(crate) use topgui::*;

#[profiling::function]
pub(crate) fn run_ui_systems(goria: &Egregoria, uiworld: &mut UiWorld) {
    bulldozer::bulldozer(goria, uiworld);
    inspected_aura::inspected_aura(goria, uiworld);
    lotbrush::lotbrush(goria, uiworld);
    roadbuild::roadbuild(goria, uiworld);
    roadeditor::roadeditor(goria, uiworld);
    selectable::selectable(goria, uiworld);
    specialbuilding::specialbuilding(goria, uiworld);
    addtrain::addtrain(goria, uiworld);
}

#[derive(Default, Clone, Debug)]
pub struct PotentialCommands(pub Vec<WorldCommand>);

impl PotentialCommands {
    pub fn set(&mut self, cmd: WorldCommand) {
        self.0.clear();
        self.0.push(cmd);
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct InspectedEntity {
    pub(crate) e: Option<Entity>,
    pub(crate) dist2: f32,
}

impl Default for InspectedEntity {
    fn default() -> Self {
        Self {
            e: None,
            dist2: 0.0,
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub(crate) enum Tool {
    Hand,
    RoadbuildStraight,
    RoadbuildCurved,
    RoadEditor,
    Bulldozer,
    LotBrush,
    SpecialBuilding,
    Train,
}

impl Tool {
    pub(crate) fn is_roadbuild(&self) -> bool {
        matches!(self, Tool::RoadbuildStraight | Tool::RoadbuildCurved)
    }

    pub fn show_arrows(&self) -> bool {
        matches!(
            self,
            Tool::RoadbuildStraight
                | Tool::RoadbuildCurved
                | Tool::RoadEditor
                | Tool::Bulldozer
                | Tool::Train
        )
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) enum UiTex {
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

#[derive(Default)]
pub(crate) struct UiTextures {
    textures: FastMap<UiTex, TextureHandle>,
}

impl UiTextures {
    pub(crate) fn new(ctx: &mut egui::Context) -> Self {
        let mut textures = common::fastmap_with_capacity(UI_TEXTURES.len());
        for &(name, path) in UI_TEXTURES {
            let (img, width, height) = wgpu_engine::Texture::read_image(path)
                .unwrap_or_else(|| panic!("Couldn't load gui texture {}", path));

            let h = ctx.load_texture(
                path,
                ImageData::Color(ColorImage::from_rgba_unmultiplied(
                    [width as usize, height as usize],
                    &img,
                )),
                TextureOptions::LINEAR,
            );

            textures.insert(name, h);
        }
        Self { textures }
    }

    pub(crate) fn get(&self, name: UiTex) -> TextureId {
        self.textures.get(&name).unwrap().id()
    }
}

impl Default for Tool {
    fn default() -> Self {
        Tool::Hand
    }
}
