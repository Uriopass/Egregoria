use super::{InspectArgsDefault, InspectRenderDefault};

impl InspectRenderDefault<u64> for u64 {
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
            args.min_value.map(|x| x as u64).unwrap_or(u64::MIN)
                ..=args.max_value.map(|x| x as u64).unwrap_or(u64::MAX),
        ));
        before != *data
    }
}
