use super::{
    get_same_or_none, get_same_or_none_mut, imgui, InspectArgsDefault, InspectRenderDefault,
};

impl InspectRenderDefault<f32> for f32 {
    fn render(data: &[&f32], label: &'static str, ui: &imgui::Ui<'_>, _args: &InspectArgsDefault) {
        if data.is_empty() {
            // Values are inconsistent
            let style_token = ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);
            ui.text(&imgui::im_str!("{}: ", label));
            style_token.pop(ui);
            return;
        }

        match get_same_or_none(data) {
            Some(_v) => {
                // Values are consistent
                let mut cp = *data[0];
                imgui::InputFloat::new(ui, &*imgui::im_str!("{}", label), &mut cp)
                    .read_only(true)
                    .build();
            }
            None => {
                // Values are inconsistent
                let style_token =
                    ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]);
                ui.text(&imgui::im_str!("{}: ", label));
                style_token.pop(ui);
            }
        }
    }

    fn render_mut(
        data: &mut [&mut f32],
        label: &'static str,
        ui: &imgui::Ui<'_>,
        _args: &InspectArgsDefault,
    ) -> bool {
        let same_or_none_value = get_same_or_none_mut(data);

        let mut value = same_or_none_value.unwrap_or(0.0);

        let style_token = if same_or_none_value.is_none() {
            // If values are inconsistent, push a style
            Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        } else {
            None
        };

        let mut changed = false;
        if ui
            .input_float(&imgui::im_str!("{}", label), &mut value)
            .build()
        {
            for d in data {
                **d = value;
                changed = true;
            }
        }

        if let Some(style_token) = style_token {
            style_token.pop(ui);
        }

        changed
    }
}
