use imgui::{StyleVar, Ui};
use serde::{Deserialize, Serialize};

use egregoria::Egregoria;

mod config;
pub mod debug;
mod map;
mod scenarios;
pub mod settings;

pub trait ImguiWindow: Send + Sync {
    fn render(&mut self, window: imgui::Window, ui: &Ui, goria: &mut Egregoria);
}

impl<F> ImguiWindow for F
where
    F: Fn(imgui::Window, &Ui, &mut Egregoria) + Send + Sync,
{
    fn render(&mut self, window: imgui::Window, ui: &Ui, goria: &mut Egregoria) {
        self(window, ui, goria);
    }
}

struct ImguiWindowStruct {
    w: Box<dyn ImguiWindow>,
    name: &'static imgui::ImStr,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct ImguiWindows {
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
        s.insert(imgui::im_str!("Map"), map::map, true);
        s.insert(
            imgui::im_str!("Scenarios"),
            scenarios::Scenarios::default(),
            false,
        );
        s.insert(imgui::im_str!("Config"), config::config, false);
        s.insert(imgui::im_str!("Debug"), debug::debug, false);
        s.insert(imgui::im_str!("Settings"), settings::settings, false);
        s
    }
}

impl ImguiWindows {
    pub fn insert(
        &mut self,
        name: &'static imgui::ImStr,
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

    pub fn menu(&mut self, ui: &Ui) {
        if self.opened.len() < self.windows.len() {
            self.opened
                .extend(std::iter::repeat(false).take(self.windows.len() - self.opened.len()))
        }
        let h = ui.window_size()[1];
        for (opened, w) in self.opened.iter_mut().zip(self.windows.iter()) {
            let tok = ui.push_style_var(StyleVar::Alpha(if *opened { 1.0 } else { 0.5 }));
            *opened ^= ui.button(w.name, [80.0, h]);
            tok.pop(ui);
        }
    }

    pub fn render(&mut self, ui: &Ui, goria: &mut Egregoria) {
        for (ws, opened) in self.windows.iter_mut().zip(self.opened.iter_mut()) {
            if *opened {
                ws.w.render(imgui::Window::new(ws.name).opened(opened), ui, goria);
            }
        }
    }
}
