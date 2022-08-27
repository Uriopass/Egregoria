use crate::{InspectArgsDefault, InspectRenderDefault};

impl InspectRenderDefault<u8> for u8 {
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
            args.min_value.map(|x| x as u8).unwrap_or(u8::MIN)
                ..=args.max_value.map(|x| x as u8).unwrap_or(u8::MAX),
        ));
        before != *data
    }
}
