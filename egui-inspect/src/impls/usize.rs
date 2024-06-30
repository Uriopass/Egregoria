use super::{Inspect, InspectArgs};

impl Inspect<usize> for usize {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgs) {
        // Values are consistent
        let mut cp = *data;
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(egui::DragValue::new(&mut cp));
        });
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        let before = *data;
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(
                egui::DragValue::new(data)
                    .range(args.min_value.unwrap_or(f32::MIN)..=args.max_value.unwrap_or(f32::MAX)),
            );
        });
        before != *data
    }
}
