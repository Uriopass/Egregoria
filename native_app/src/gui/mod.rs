use common::FastMap;
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

use egui::{ColorImage, ImageData, TextureHandle, TextureId, TextureOptions};
use engine::yakui::YakuiWrapper;
use engine::GfxContext;
use serde::{Deserialize, Serialize};

use crate::uiworld::UiWorld;
use roadbuild::RoadBuildResource;
use simulation::map::BuildingID;
use simulation::world_command::WorldCommand;
use simulation::{AnyEntity, Simulation};

pub mod chat;
pub mod follow;
pub mod inspect;
pub mod inspected_aura;
mod tools;
pub mod topgui;
pub mod windows;

pub use follow::FollowEntity;
pub use tools::*;
pub use topgui::*;

pub fn run_ui_systems(sim: &Simulation, uiworld: &mut UiWorld) {
    profiling::scope!("gui::run_ui_systems");
    bulldozer::bulldozer(sim, uiworld);
    inspected_aura::inspected_aura(sim, uiworld);
    lotbrush::lotbrush(sim, uiworld);
    roadbuild::roadbuild(sim, uiworld);
    roadeditor::roadeditor(sim, uiworld);
    specialbuilding::specialbuilding(sim, uiworld);
    addtrain::addtrain(sim, uiworld);
    zoneedit::zoneedit(sim, uiworld);
    terraforming::terraforming(sim, uiworld);

    // run last so other systems can have the chance to cancel select
    selectable::selectable(sim, uiworld);
}

#[derive(Default, Clone, Debug)]
pub struct ErrorTooltip {
    pub msg: Option<Cow<'static, str>>,
    // Whether this tooltip is about something happening in the game world
    // Avoid showing tooltip when the UI is hovered
    pub isworld: bool,
}

impl ErrorTooltip {
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            msg: Some(msg.into()),
            isworld: true,
        }
    }

    #[allow(unused)]
    pub fn new_ui(msg: impl Into<Cow<'static, str>>) -> Self {
        Self {
            msg: Some(msg.into()),
            isworld: false,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct PotentialCommands(pub Vec<WorldCommand>);

impl PotentialCommands {
    pub fn set(&mut self, cmd: WorldCommand) {
        self.0.clear();
        self.0.push(cmd);
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct InspectedBuilding {
    pub e: Option<BuildingID>,
    pub dontclear: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct InspectedEntity {
    pub e: Option<AnyEntity>,
    pub dist2: f32,
    pub dontclear: bool,
}

impl Default for InspectedEntity {
    fn default() -> Self {
        Self {
            e: None,
            dist2: 0.0,
            dontclear: false,
        }
    }
}

/// Time that always progresses even when the game is paused
/// Is moduloed to 3600
#[derive(Copy, Clone, Debug, Default)]
pub struct TimeAlways(pub f32);

#[derive(Copy, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
pub enum Tool {
    #[default]
    Hand,
    RoadbuildStraight,
    RoadbuildCurved,
    RoadEditor,
    Bulldozer,
    LotBrush,
    SpecialBuilding,
    Train,
    Terraforming,
}

impl Tool {
    pub fn is_roadbuild(&self) -> bool {
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

    pub fn show_lots(&self) -> bool {
        matches!(
            self,
            Tool::RoadbuildStraight | Tool::RoadbuildCurved | Tool::Bulldozer | Tool::LotBrush
        )
    }
}

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
        self.textures.get(name).unwrap().id()
    }

    pub fn get_yakui(&self, name: &str) -> yakui::TextureId {
        match self.yakui_textures.get(name) {
            None => panic!("Couldn't find texture {}", name),
            Some(x) => *x,
        }
    }

    pub fn try_get(&self, name: &str) -> Option<TextureId> {
        self.textures.get(name).map(TextureHandle::id)
    }
}
