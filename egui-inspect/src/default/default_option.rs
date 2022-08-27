use super::{InspectArgsDefault, InspectRenderDefault};

impl<T: InspectRenderDefault<T>> InspectRenderDefault<Option<T>> for Option<T> {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgsDefault) {
        match data {
            Some(value) => <T as InspectRenderDefault<T>>::render(value, label, ui, args),
            None => {
                ui.label(&format!("{}: None", label));
            }
        };
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        match data {
            Some(value) => <T as InspectRenderDefault<T>>::render_mut(value, label, ui, args),
            None => {
                ui.label(&format!("{}: None", label));
                false
            }
        }
    }
}
