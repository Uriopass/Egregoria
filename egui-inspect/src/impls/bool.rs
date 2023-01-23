use super::{Inspect, InspectArgs};

impl Inspect<bool> for bool {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgs) {
        ui.label(&format!("{label}: {data}"));
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgs,
    ) -> bool {
        ui.checkbox(data, label).changed()
    }
}
