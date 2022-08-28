use crate::uiworld::UiWorld;
use common::Config;
use egregoria::Egregoria;
use egui_inspect::{InspectArgsDefault, InspectRenderDefault};

pub(crate) fn config(window: egui::Window<'_>, ui: &egui::Context, _: &mut UiWorld, _: &Egregoria) {
    window.default_size([600.0, 500.0]).show(ui, |ui| {
        let mut config = (**common::config()).clone();

        let args = InspectArgsDefault {
            header: Some(false),
            indent_children: Some(false),
            ..InspectArgsDefault::default()
        };
        if <Config as InspectRenderDefault<Config>>::render_mut(&mut config, "", ui, &args) {
            common::update_config(config);
        }
    });
}
