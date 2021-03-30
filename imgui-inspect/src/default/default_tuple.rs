use super::*;

impl<A: InspectRenderDefault<A>, B: InspectRenderDefault<B>> InspectRenderDefault<(A, B)>
    for (A, B)
{
    fn render(data: &[&(A, B)], _: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        let (a, b) = data[0];
        <A as InspectRenderDefault<A>>::render(&[a], "0", ui, args);
        <B as InspectRenderDefault<B>>::render(&[b], "1", ui, args);
    }

    fn render_mut(
        data: &mut [&mut (A, B)],
        _: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let (a, b) = data[0];
        let mut changed = false;
        changed |= <A as InspectRenderDefault<A>>::render_mut(&mut [a], "0", ui, args);
        changed |= <B as InspectRenderDefault<B>>::render_mut(&mut [b], "1", ui, args);
        changed
    }
}
