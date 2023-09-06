use crate::uiworld::UiWorld;
use egui_inspect::{Inspect, InspectArgs};
use simulation::Config;
use simulation::Simulation;

/// Config window
/// Allows to change the real-time dev-only config
pub fn config(window: egui::Window<'_>, ui: &egui::Context, _: &mut UiWorld, _: &Simulation) {
    window
        .default_size([600.0, 500.0])
        .vscroll(true)
        .show(ui, |ui| {
            let mut config = (**simulation::config()).clone();

            let args = InspectArgs {
                header: Some(false),
                indent_children: Some(false),
                ..InspectArgs::default()
            };
            if <Config as Inspect<Config>>::render_mut(&mut config, "", ui, &args) {
                simulation::update_config(config);
            }
        });
}
