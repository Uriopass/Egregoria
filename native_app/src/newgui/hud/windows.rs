use crate::gui::Gui;
use crate::uiworld::UiWorld;
use goryak::Window;
use simulation::Simulation;

#[derive(Default)]
pub struct Windows {
    pub windows: Vec<(Window, Box<dyn FnOnce(&mut Gui, &UiWorld, &Simulation)>)>,
}

impl Windows {
    pub fn show(
        &mut self,
        window: Window,
        children: impl FnOnce(&mut Gui, &UiWorld, &Simulation) + 'static,
    ) {
        self.windows.push((window, Box::new(children)));
    }

    pub fn finish(gui: &mut Gui, uiw: &UiWorld, sim: &Simulation) {
        let mut window_state = uiw.write::<Windows>();
        let windows = std::mem::take(&mut window_state.windows);
        window_state.windows = Vec::with_capacity(windows.len());
        drop(window_state);

        for (window, children) in windows {
            window.show(|| {
                children(gui, uiw, sim);
            });
        }
    }
}
