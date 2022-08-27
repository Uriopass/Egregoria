use super::{InspectArgsDefault, InspectRenderDefault};

impl InspectRenderDefault<String> for String {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgsDefault) {
        ui.label(&format!("{}: {}", label, data));
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            changed = ui.text_edit_singleline(data).changed();
            ui.label(label);
        });
        changed
    }
}
