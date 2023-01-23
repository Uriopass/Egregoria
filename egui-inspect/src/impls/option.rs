use super::{Inspect, InspectArgs};

impl<T: Inspect<T>> Inspect<Option<T>> for Option<T> {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs) {
        match data {
            Some(value) => <T as Inspect<T>>::render(value, label, ui, args),
            None => {
                ui.label(&format!("{label}: None"));
            }
        };
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        match data {
            Some(value) => <T as Inspect<T>>::render_mut(value, label, ui, args),
            None => {
                ui.label(&format!("{label}: None"));
                false
            }
        }
    }
}
