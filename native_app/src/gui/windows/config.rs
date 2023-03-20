use crate::uiworld::UiWorld;
use egregoria::Config;
use egregoria::Egregoria;
use egui_inspect::{Inspect, InspectArgs};

/// Config window
/// Allows to change the real-time dev-only config
pub(crate) fn config(window: egui::Window<'_>, ui: &egui::Context, _: &mut UiWorld, _: &Egregoria) {
    window
        .default_size([600.0, 500.0])
        .vscroll(true)
        .show(ui, |ui| {
            let mut config = (**egregoria::config()).clone();

            let args = InspectArgs {
                header: Some(false),
                indent_children: Some(false),
                ..InspectArgs::default()
            };
            if <Config as Inspect<Config>>::render_mut(&mut config, "", ui, &args) {
                egregoria::update_config(config);
            }
        });
}
