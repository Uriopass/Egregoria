use crate::{InspectArgsDefault, InspectRenderDefault};
use imgui::{im_str, Ui};

impl<T: InspectRenderDefault<T>> InspectRenderDefault<Vec<T>> for Vec<T> {
    fn render(data: &[&Vec<T>], label: &'static str, ui: &Ui<'_>, args: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = data[0];

        if imgui::CollapsingHeader::new(&im_str!("{}", label)).build(ui) {
            ui.indent();
            for (i, x) in v.iter().enumerate() {
                let id = ui.push_id(i as i32);
                <T as InspectRenderDefault<T>>::render(&[x], "", ui, args);
                id.pop(ui);
            }
            ui.unindent();
        }
    }

    fn render_mut(
        data: &mut [&mut Vec<T>],
        label: &str,

        ui: &Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &mut data[0];

        let mut changed = false;
        if imgui::CollapsingHeader::new(&im_str!("{}", label)).build(ui) {
            ui.indent();
            for (i, x) in v.iter_mut().enumerate() {
                let id = ui.push_id(i as i32);
                changed |= <T as InspectRenderDefault<T>>::render_mut(&mut [x], "", ui, args);
                id.pop(ui);
            }
            ui.unindent();
        }

        changed
    }
}
