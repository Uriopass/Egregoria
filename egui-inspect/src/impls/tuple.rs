use crate::{Inspect, InspectArgs};

impl<A: Inspect<A>, B: Inspect<B>> Inspect<(A, B)> for (A, B) {
    fn render((a, b): &(A, B), label: &'static str, ui: &mut egui::Ui, args: &InspectArgs) {
        ui.indent(label, |ui| {
            <A as Inspect<A>>::render(a, "", ui, args);
            <B as Inspect<B>>::render(b, "", ui, args);
        });
        ui.separator();
    }

    fn render_mut(
        (a, b): &mut (A, B),
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        let mut changed = false;
        ui.indent(label, |ui| {
            changed |= <A as Inspect<A>>::render_mut(a, "", ui, args);
            changed |= <B as Inspect<B>>::render_mut(b, "", ui, args);
        });
        ui.separator();
        changed
    }
}
