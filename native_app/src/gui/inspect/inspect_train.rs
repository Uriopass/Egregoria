use crate::uiworld::UiWorld;
use egregoria::{Egregoria, TrainID};
use egui::Context;

pub fn inspect_train(_uiworld: &mut UiWorld, goria: &Egregoria, ui: &Context, id: TrainID) -> bool {
    let Some(t) = goria.get(id) else { return false; };

    let mut is_open = true;
    egui::Window::new("Train")
        .resizable(false)
        .auto_sized()
        .open(&mut is_open)
        .show(ui, |ui| {
            if cfg!(debug_assertions) {
                ui.label(format!("{:?}", id));
            }

            ui.label(format!("Going at {:.0}km/h", t.speed.0));
        });

    is_open
}
