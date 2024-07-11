use crate::gui::windows::GUIWindows;
use crate::uiworld::UiWorld;
use serde::{Deserialize, Serialize};
use simulation::map::BuildingID;
use simulation::world_command::WorldCommand;
use simulation::{AnyEntity, Simulation};
use std::borrow::Cow;
use std::time::Instant;

pub mod follow;
mod hud;
pub mod inspect;
mod textures;
mod tools;

pub use hud::*;
pub use textures::*;
pub use tools::*;

pub struct GuiState {
    pub debug_window: bool,
    pub windows: GUIWindows,
    pub last_save: Instant,
    pub depause_warp: u32,
    pub hidden: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            debug_window: false,
            windows: Default::default(),
            last_save: Instant::now(),
            depause_warp: 1,
            hidden: false,
        }
    }
}

pub fn run_ui_systems(sim: &Simulation, uiworld: &UiWorld) {
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

pub enum ExitState {
    NoExit,
    ExitAsk,
    Saving,
}

impl Default for ExitState {
    fn default() -> Self {
        Self::NoExit
    }
}
