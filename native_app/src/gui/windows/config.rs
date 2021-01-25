use common::Config;
use egregoria::Egregoria;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};

pub fn config(ui: &Ui, _goria: &mut Egregoria) {
    let mut config = (**common::config()).clone();

    let args = InspectArgsDefault {
        header: Some(false),
        indent_children: Some(false),
        ..InspectArgsDefault::default()
    };
    if <Config as InspectRenderDefault<Config>>::render_mut(&mut [&mut config], "", ui, &args) {
        common::update_config(config);
    }
}
