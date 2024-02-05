use egui::Context;
use goryak::button_primary;

use crate::inputmap::{InputAction, InputMap};
use crate::uiworld::UiWorld;
use simulation::Simulation;

pub mod debug;
pub mod load;
#[cfg(feature = "multiplayer")]
pub mod network;
pub mod settings;

pub trait GUIWindow: Send + Sync {
    fn render_window(
        &mut self,
        window: egui::Window<'_>,
        ui: &Context,
        uiworld: &UiWorld,
        sim: &Simulation,
    );
}

impl<F> GUIWindow for F
where
    F: Fn(egui::Window<'_>, &Context, &UiWorld, &Simulation) + Send + Sync,
{
    fn render_window(
        &mut self,
        window: egui::Window<'_>,
        ui: &Context,
        uiworld: &UiWorld,
        sim: &Simulation,
    ) {
        self(window, ui, uiworld, sim);
    }
}

struct GUIWindowStruct {
    w: Box<dyn GUIWindow>,
    name: &'static str,
}

pub struct OldGUIWindows {
    windows: Vec<GUIWindowStruct>,
    opened: Vec<bool>,
}

impl Default for OldGUIWindows {
    fn default() -> Self {
        let mut s = Self {
            windows: vec![],
            opened: vec![],
        };
        s.insert("Debug", debug::debug, false);
        s.insert("Settings", settings::settings, false);
        #[cfg(feature = "multiplayer")]
        s.insert("Network", network::network, false);
        s.insert("Load", load::load, false);
        s
    }
}

impl OldGUIWindows {
    pub fn insert(&mut self, name: &'static str, w: impl GUIWindow + 'static, opened: bool) {
        self.windows.push(GUIWindowStruct {
            w: Box::new(w),
            name,
        });
        if self.opened.len() < self.windows.len() {
            self.opened.push(opened)
        }
    }

    pub fn menu(&mut self) {
        if self.opened.len() < self.windows.len() {
            self.opened
                .extend(std::iter::repeat(false).take(self.windows.len() - self.opened.len()))
        }
        for (opened, w) in self.opened.iter_mut().zip(self.windows.iter()) {
            *opened ^= button_primary(w.name).show().clicked;
        }
    }

    pub fn render(&mut self, ui: &Context, uiworld: &UiWorld, sim: &Simulation) {
        profiling::scope!("windows::render");
        if uiworld
            .write::<InputMap>()
            .just_act
            .contains(&InputAction::OpenEconomyMenu)
        {
            for (i, w) in self.windows.iter().enumerate() {
                if w.name == "Economy" {
                    self.opened[i] ^= true;
                }
            }
        }
        for (ws, opened) in self.windows.iter_mut().zip(self.opened.iter_mut()) {
            if *opened {
                ws.w.render_window(egui::Window::new(ws.name).open(opened), ui, uiworld, sim);
            }
        }
    }
}
