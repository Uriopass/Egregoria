pub mod economy;
pub mod load;
pub mod settings;

use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;
use goryak::button_primary;
use simulation::Simulation;

#[cfg(feature = "multiplayer")]
pub mod network;

#[derive(Default)]
pub struct GUIWindows {
    economy_open: bool,
    settings_open: bool,
    load_open: bool,
    #[cfg(feature = "multiplayer")]
    network_open: bool,
}

impl GUIWindows {
    pub fn menu(&mut self) {
        if button_primary("Economy").show().clicked {
            self.economy_open ^= true;
        }

        if button_primary("Settings").show().clicked {
            self.settings_open ^= true;
        }

        if button_primary("Load").show().clicked {
            self.load_open ^= true;
        }

        #[cfg(feature = "multiplayer")]
        if button_primary("Network").show().clicked {
            self.network_open ^= true;
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

        economy::economy(uiworld, sim, &mut self.economy_open);
        settings::settings(uiworld, sim, &mut self.settings_open);
        load::load(uiworld, sim, &mut self.load_open);

        #[cfg(feature = "multiplayer")]
        network::network(uiworld, sim, &mut self.network_open);
    }
}
