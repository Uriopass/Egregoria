use super::{InspectArgsDefault, InspectRenderDefault};

impl InspectRenderDefault<bool> for bool {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgsDefault) {
        ui.label(&format!("{}: {}", label, data));
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        ui.checkbox(data, label).changed()
    }
}
