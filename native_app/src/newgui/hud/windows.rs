use crate::gui::Gui;
use crate::uiworld::UiWorld;
use goryak::Window;
use simulation::Simulation;

#[derive(Default)]
pub struct Windows {
    windows: Vec<WindowHolder>,
}

struct WindowHolder {
    window: Window,
    children: Box<dyn FnOnce(&mut Gui, &UiWorld, &Simulation)>,
    on_close: Box<dyn FnOnce(&UiWorld)>,
}

impl Windows {
    pub fn show(
        &mut self,
        window: Window,
        children: impl FnOnce(&mut Gui, &UiWorld, &Simulation) + 'static,
        on_close: impl FnOnce(&UiWorld) + 'static,
    ) {
        self.windows.push(WindowHolder {
            window,
            children: Box::new(children),
            on_close: Box::new(on_close),
        });
    }

    pub fn finish(gui: &mut Gui, uiw: &UiWorld, sim: &Simulation) {
        let mut window_state = uiw.write::<Windows>();
        let windows = std::mem::take(&mut window_state.windows);
        window_state.windows = Vec::with_capacity(windows.len());
        drop(window_state);

        for WindowHolder {
            window,
            children,
            on_close,
        } in windows
        {
            window.show(
                || {
                    children(gui, uiw, sim);
                },
                || {
                    on_close(uiw);
                },
            );
        }
    }
}
