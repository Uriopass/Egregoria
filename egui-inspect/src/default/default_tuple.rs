use crate::{InspectArgsDefault, InspectRenderDefault};

impl<A: InspectRenderDefault<A>, B: InspectRenderDefault<B>> InspectRenderDefault<(A, B)>
    for (A, B)
{
    fn render((a, b): &(A, B), label: &'static str, ui: &mut egui::Ui, args: &InspectArgsDefault) {
        ui.indent(label, |ui| {
            <A as InspectRenderDefault<A>>::render(a, "", ui, args);
            <B as InspectRenderDefault<B>>::render(b, "", ui, args);
        });
        ui.separator();
    }

    fn render_mut(
        (a, b): &mut (A, B),
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let mut changed = false;
        ui.indent(label, |ui| {
            changed |= <A as InspectRenderDefault<A>>::render_mut(a, "", ui, args);
            changed |= <B as InspectRenderDefault<B>>::render_mut(b, "", ui, args);
        });
        ui.separator();
        changed
    }
}
