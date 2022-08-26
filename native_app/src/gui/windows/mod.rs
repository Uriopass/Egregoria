use imgui::{StyleVar, Ui};
use serde::{Deserialize, Serialize};

use crate::uiworld::UiWorld;
use egregoria::Egregoria;

mod config;
pub(crate) mod debug;
mod economy;
#[cfg(feature = "multiplayer")]
pub(crate) mod network;
pub(crate) mod settings;

pub(crate) trait ImguiWindow: Send + Sync {
    fn render_window(
        &mut self,
        window: imgui::Window<'_, &'static str>,
        ui: &Ui<'_>,
        uiworld: &mut UiWorld,
        goria: &Egregoria,
    );
}

impl<F> ImguiWindow for F
where
    F: Fn(imgui::Window<'_, &'static str>, &Ui<'_>, &mut UiWorld, &Egregoria) + Send + Sync,
{
    fn render_window(
        &mut self,
        window: imgui::Window<'_, &'static str>,
        ui: &Ui<'_>,
        uiworld: &mut UiWorld,
        goria: &Egregoria,
    ) {
        self(window, ui, uiworld, goria);
    }
}

struct ImguiWindowStruct {
    w: Box<dyn ImguiWindow>,
    name: &'static str,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct ImguiWindows {
    #[serde(skip)]
    windows: Vec<ImguiWindowStruct>,
    opened: Vec<bool>,
}

impl Default for ImguiWindows {
    fn default() -> Self {
        let mut s = Self {
            windows: vec![],
            opened: vec![],
        };
        s.insert("Economy", economy::economy, false);
        s.insert("Config", config::config, false);
        s.insert("Debug", debug::debug, false);
        s.insert("Settings", settings::settings, false);
        #[cfg(feature = "multiplayer")]
        s.insert("Network", network::network, false);
        s
    }
}

impl ImguiWindows {
    pub(crate) fn insert(
        &mut self,
        name: &'static str,
        w: impl ImguiWindow + 'static,
        opened: bool,
    ) {
        self.windows.push(ImguiWindowStruct {
            w: Box::new(w),
            name,
        });
        if self.opened.len() < self.windows.len() {
            self.opened.push(opened)
        }
    }

    pub(crate) fn menu(&mut self, ui: &Ui<'_>) {
        if self.opened.len() < self.windows.len() {
            self.opened
                .extend(std::iter::repeat(false).take(self.windows.len() - self.opened.len()))
        }
        let h = ui.window_size()[1];
        for (opened, w) in self.opened.iter_mut().zip(self.windows.iter()) {
            let tok = ui.push_style_var(StyleVar::Alpha(if *opened { 1.0 } else { 0.5 }));
            *opened ^= ui.button_with_size(w.name, [80.0, h]);
            tok.pop();
        }
    }

    pub(crate) fn render(&mut self, ui: &Ui<'_>, uiworld: &mut UiWorld, goria: &Egregoria) {
        for (ws, opened) in self.windows.iter_mut().zip(self.opened.iter_mut()) {
            if *opened {
                ws.w.render_window(
                    imgui::Window::new(ws.name).opened(opened),
                    ui,
                    uiworld,
                    goria,
                );
            }
        }
    }
}
