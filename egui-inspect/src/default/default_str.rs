use super::{InspectArgsDefault, InspectRenderDefault};

impl InspectRenderDefault<&'static str> for &'static str {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgsDefault) {
        // Values are consistent
        ui.label(&format!("{}: {}", label, data));
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        Self::render(data, label, ui, _args);
        false
    }
}
