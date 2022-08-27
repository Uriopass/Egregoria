use crate::{InspectArgsDefault, InspectRenderDefault};

impl<T: InspectRenderDefault<T>> InspectRenderDefault<Vec<T>> for Vec<T> {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgsDefault) {
        <[T] as InspectRenderDefault<[T]>>::render(data, label, ui, args);
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        <[T] as InspectRenderDefault<[T]>>::render_mut(data, label, ui, args)
    }
}

impl<T: InspectRenderDefault<T>> InspectRenderDefault<[T]> for [T] {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgsDefault) {
        ui.collapsing(label, |ui| {
            for (i, x) in data.iter().enumerate() {
                ui.push_id(i, |ui| {
                    <T as InspectRenderDefault<T>>::render(x, "", ui, args);
                });
            }
        });
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let mut changed = false;

        ui.collapsing(label, |ui| {
            for (i, x) in data.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    changed |= <T as InspectRenderDefault<T>>::render_mut(x, "", ui, args);
                });
            }
        });
        changed
    }
}
