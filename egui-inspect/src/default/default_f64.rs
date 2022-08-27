use super::{InspectArgsDefault, InspectRenderDefault};

impl InspectRenderDefault<f64> for f64 {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgsDefault) {
        // Values are consistent
        let mut cp = *data;
        ui.add(egui::DragValue::new(&mut cp).suffix(label));
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let before = *data;
        ui.add(egui::DragValue::new(data).suffix(label).clamp_range(
            args.min_value.unwrap_or(f32::MIN) as f64..=args.max_value.unwrap_or(f32::MAX) as f64,
        ));
        before != *data
    }
}
