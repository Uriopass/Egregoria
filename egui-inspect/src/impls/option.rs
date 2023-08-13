use super::{Inspect, InspectArgs};

pub struct OptionDefault;

impl<T: Inspect<T> + Default> Inspect<Option<T>> for OptionDefault {
    fn render(data: &Option<T>, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs) {
        match data {
            Some(value) => <T as Inspect<T>>::render(value, label, ui, args),
            None => {
                ui.label(&format!("{label}: None"));
            }
        };
    }

    fn render_mut(
        data: &mut Option<T>,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        let mut changed = false;
        let mut is_some = data.is_some();
        if ui.checkbox(&mut is_some, label).changed() {
            changed = true;
            if is_some {
                *data = Some(T::default());
            } else {
                *data = None;
            }
        }

        let mut args = args.clone();
        args.header = Some(false);

        match data {
            Some(value) => changed |= <T as Inspect<T>>::render_mut(value, label, ui, &args),
            None => {}
        }

        changed
    }
}

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
