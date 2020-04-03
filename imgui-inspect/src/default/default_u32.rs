use super::*;

impl InspectRenderDefault<u32> for u32 {
    fn render(
        data: &[&u32],
        label: &'static str,
        _: &mut World,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        if data.len() == 0 {
            // Values are inconsistent
            let style_token = ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);
            ui.text(&imgui::im_str!("{}: ", label));
            style_token.pop(ui);
            return;
        }

        match get_same_or_none(data) {
            Some(_v) => {
                // Values are consistent
                ui.text(&imgui::im_str!("{}: {}", label, data[0]))
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
        data: &mut [&mut u32],
        label: &'static str,
        _: &mut World,
        ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let same_or_none_value = get_same_or_none_mut(data);

        let value = match same_or_none_value {
            Some(v) => v,
            None => 0, // Some reasonable default
        };

        // CAST
        let mut value = value as i32;

        let style_token = if same_or_none_value.is_none() {
            // If values are inconsistent, push a style
            Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        } else {
            None
        };

        let mut changed = false;
        if ui
            .input_int(&imgui::im_str!("{}", label), &mut value)
            .build()
        {
            for d in data {
                // CAST
                let value = value as u32;

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
