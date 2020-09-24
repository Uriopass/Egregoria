mod info;
mod map;
mod scenarios;
mod tips;

use egregoria::Egregoria;
use imgui::Ui;

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
    opened: bool,
}

pub struct ImguiWindows {
    windows: Vec<ImguiWindowStruct>,
}

impl Default for ImguiWindows {
    fn default() -> Self {
        let mut s = Self { windows: vec![] };
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
            opened,
        })
    }

    pub fn menu(&mut self, ui: &Ui) {
        ui.menu(imgui::im_str!("Show"), true, || {
            for v in &mut self.windows {
                v.opened |= imgui::MenuItem::new(v.name).build(ui);
            }
        });
    }

    pub fn render(&mut self, ui: &Ui, goria: &mut Egregoria) {
        for v in &mut self.windows {
            if v.opened {
                let w = &mut v.w;
                imgui::Window::new(v.name)
                    .opened(&mut v.opened)
                    .build(&ui, || {
                        w.render(ui, goria);
                    });
            }
        }
    }
}
