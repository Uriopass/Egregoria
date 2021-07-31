use super::*;

impl<T: InspectRenderDefault<T>> InspectRenderDefault<Option<T>> for Option<T> {
    fn render(
        data: &[&Option<T>],
        label: &'static str,
        ui: &imgui::Ui<'_>,
        args: &InspectArgsDefault,
    ) {
        if data.is_empty() {
            ui.text(&imgui::im_str!("{}: None", label));
            return;
        }

        let d = data[0];
        match d {
            Some(value) => <T as InspectRenderDefault<T>>::render(&[value], label, ui, args),
            None => ui.text(&imgui::im_str!("{}: None", label)),
        };
    }

    fn render_mut(
        data: &mut [&mut Option<T>],
        label: &'static str,
        ui: &imgui::Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.is_empty() {
            ui.text(&imgui::im_str!("{}: None", label));
            return false;
        }

        let d = &mut data[0];
        match d {
            Some(value) => {
                <T as InspectRenderDefault<T>>::render_mut(&mut [value], label, ui, args)
            }
            None => {
                ui.text(&imgui::im_str!("{}: None", label));
                false
            }
        }
    }
}
