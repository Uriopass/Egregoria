use crate::gui::inspect::follow_button;
use crate::uiworld::UiWorld;
use egui::Context;
use simulation::{Simulation, TrainID};

pub fn inspect_train(uiworld: &UiWorld, sim: &Simulation, ui: &Context, id: TrainID) -> bool {
    let Some(t) = sim.get(id) else {
        return false;
    };

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

            follow_button(uiworld, ui, id);
        });

    is_open
}
