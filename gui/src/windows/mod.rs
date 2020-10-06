mod info;
mod map;
mod scenarios;
mod tips;

use egregoria::Egregoria;
use imgui::Ui;
use serde::{Deserialize, Serialize};

pub trait ImguiWindow: Send + Sync {
    fn render(&mut self, ui: &Ui, goria: &mut Egregoria);
}

impl<F> ImguiWindow for F
where
    F: Fn(&Ui, &mut Egregoria) + Send + Sync,
{
    fn render(&mut self, ui: &Ui, goria: &mut Egregoria) {
        self(ui, goria);
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
        s.insert(imgui::im_str!("Infos"), info::info, false);
        s.insert(imgui::im_str!("Map"), map::map, true);
        s.insert(
            imgui::im_str!("Scenarios"),
            scenarios::Scenarios::default(),
            false,
        );
        s.insert(imgui::im_str!("Tips"), tips::tips, false);
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
        ui.menu(imgui::im_str!("Show"), true, || {
            for (opened, w) in self.opened.iter_mut().zip(self.windows.iter()) {
                *opened |= imgui::MenuItem::new(w.name).build(ui);
            }
        });
    }

    pub fn render(&mut self, ui: &Ui, goria: &mut Egregoria) {
        for (ws, opened) in self.windows.iter_mut().zip(self.opened.iter_mut()) {
            if *opened {
                imgui::Window::new(ws.name).opened(opened).build(&ui, || {
                    ws.w.render(ui, goria);
                });
            }
        }
    }
}
