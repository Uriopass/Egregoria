use super::{Inspect, InspectArgs};

impl Inspect<&'static str> for &'static str {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgs) {
        // Values are consistent
        ui.label(&format!("{label}: {data}"));
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgs,
    ) -> bool {
        Self::render(data, label, ui, _args);
        false
    }
}
