use super::{imgui, InspectArgsDefault, InspectRenderDefault};

impl InspectRenderDefault<i32> for i32 {
    fn render(data: &[&i32], label: &'static str, ui: &imgui::Ui<'_>, _args: &InspectArgsDefault) {
        if !label.is_empty() {
            ui.text(&format!("{}: {}", label, data[0]))
        } else {
            ui.text(&format!("{}", data[0]))
        }
    }

    fn render_mut(
        data: &mut [&mut i32],
        label: &'static str,
        ui: &imgui::Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        let mut changed = false;
        let mut value = *data[0];

        if imgui::InputInt::new(ui, &label, &mut value).build()
            && value >= args.min_value.map(|x| x as i32).unwrap_or(i32::MIN)
            && value <= args.max_value.map(|x| x as i32).unwrap_or(i32::MAX)
        {
            *data[0] = value;
            changed = true;
        }
        changed
    }
}
