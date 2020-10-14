use common::Config;
use egregoria::Egregoria;
use imgui::Ui;
use imgui_inspect::{InspectArgsDefault, InspectRenderDefault};

pub fn config(ui: &Ui, _goria: &mut Egregoria) {
    let mut config = (**common::config()).clone();

    let mut args = InspectArgsDefault::default();
    args.header = Some(false);
    args.indent_children = Some(false);
    if <Config as InspectRenderDefault<Config>>::render_mut(&mut [&mut config], "", ui, &args) {
        common::update_config(config);
    }
}
