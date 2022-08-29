use crate::{Inspect, InspectArgs};

impl<T: Inspect<T>> Inspect<Vec<T>> for Vec<T> {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs) {
        <[T] as Inspect<[T]>>::render(data, label, ui, args);
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        <[T] as Inspect<[T]>>::render_mut(data, label, ui, args)
    }
}

impl<T: Inspect<T>> Inspect<[T]> for [T] {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs) {
        ui.collapsing(format!("{} [{}]", label, data.len()), |ui| {
            for (i, x) in data.iter().enumerate() {
                ui.push_id(i, |ui| {
                    <T as Inspect<T>>::render(x, "", ui, args);
                });
            }
        });
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        let mut changed = false;

        ui.collapsing(format!("{} [{}]", label, data.len()), |ui| {
            for (i, x) in data.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    changed |= <T as Inspect<T>>::render_mut(x, "", ui, args);
                });
            }
        });
        changed
    }
}
