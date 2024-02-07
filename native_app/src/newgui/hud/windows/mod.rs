pub mod economy;
pub mod settings;

use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;
use goryak::button_primary;
use simulation::Simulation;

#[derive(Default)]
pub struct GUIWindows {
    economy_open: bool,
    settings_open: bool,
}

impl GUIWindows {
    pub fn menu(&mut self) {
        if button_primary("Economy").show().clicked {
            self.economy_open ^= true;
        }

        if button_primary("Settings").show().clicked {
            self.settings_open ^= true;
        }
    }

    pub fn render(&mut self, uiworld: &UiWorld, sim: &Simulation) {
        profiling::scope!("windows::render");
        if uiworld
            .write::<InputMap>()
            .just_act
            .contains(&InputAction::OpenEconomyMenu)
        {
            self.economy_open ^= true;
        }

        if self.economy_open {
            economy::economy(uiworld, sim);
        }

        if self.settings_open {
            settings::settings(uiworld, sim);
        }
    }
}
