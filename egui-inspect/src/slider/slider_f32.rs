use super::InspectArgsSlider;
use crate::InspectRenderSlider;

impl InspectRenderSlider<f32> for f32 {
    fn render(v: &f32, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgsSlider) {
        ui.label(&format!("{}: {}", label, v));
    }

    fn render_mut(
        value: &mut f32,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsSlider,
    ) -> bool {
        let min = args.min_value.unwrap_or(-100.0);
        let max = args.max_value.unwrap_or(100.0);
        ui.add(egui::Slider::new(value, min..=max).text(label))
            .changed()
    }
}
