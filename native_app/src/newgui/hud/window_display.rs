use crate::uiworld::UiWorld;
use goryak::Window;
use simulation::Simulation;

#[derive(Default)]
pub struct WindowDisplay {
    windows: Vec<WindowDisplayHolder>,
}

struct WindowDisplayHolder {
    window: Window,
    children: Box<dyn FnOnce(&UiWorld, &Simulation)>,
    on_close: Box<dyn FnOnce(&UiWorld)>,
}

impl WindowDisplay {
    pub fn show(
        &mut self,
        window: Window,
        children: impl FnOnce(&UiWorld, &Simulation) + 'static,
        on_close: impl FnOnce(&UiWorld) + 'static,
    ) {
        self.windows.push(WindowDisplayHolder {
            window,
            children: Box::new(children),
            on_close: Box::new(on_close),
        });
    }

    pub fn finish(uiw: &UiWorld, sim: &Simulation) {
        let mut window_state = uiw.write::<WindowDisplay>();
        let windows = std::mem::take(&mut window_state.windows);
        window_state.windows = Vec::with_capacity(windows.len());
        drop(window_state);

        for WindowDisplayHolder {
            window,
            children,
            on_close,
        } in windows
        {
            window.show(
                || {
                    children(uiw, sim);
                },
                || {
                    on_close(uiw);
                },
            );
        }
    }
}
