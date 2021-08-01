use super::{imgui, InspectArgsDefault, InspectRenderDefault};

impl<A: InspectRenderDefault<A>, B: InspectRenderDefault<B>> InspectRenderDefault<(A, B)>
    for (A, B)
{
    fn render(data: &[&(A, B)], _: &'static str, ui: &imgui::Ui<'_>, args: &InspectArgsDefault) {
        let (a, b) = data[0];
        ui.indent();
        <A as InspectRenderDefault<A>>::render(&[a], "", ui, args);
        <B as InspectRenderDefault<B>>::render(&[b], "", ui, args);
        ui.unindent();
        ui.separator();
    }

    fn render_mut(
        data: &mut [&mut (A, B)],
        _: &'static str,
        ui: &imgui::Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        let (a, b) = data[0];
        let mut changed = false;
        ui.indent();
        changed |= <A as InspectRenderDefault<A>>::render_mut(&mut [a], "", ui, args);
        changed |= <B as InspectRenderDefault<B>>::render_mut(&mut [b], "", ui, args);
        ui.unindent();
        ui.separator();
        changed
    }
}
