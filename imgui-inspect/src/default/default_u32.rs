use super::{
    get_same_or_none, get_same_or_none_mut, imgui, InspectArgsDefault, InspectRenderDefault,
};

impl InspectRenderDefault<u32> for u32 {
    fn render(data: &[&u32], label: &'static str, ui: &imgui::Ui<'_>, _args: &InspectArgsDefault) {
        if data.is_empty() {
            // Values are inconsistent
            let style_token = ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);
            ui.text(&format!("{}: ", label));
            style_token.pop();
            return;
        }

        match get_same_or_none(data) {
            Some(_v) => {
                // Values are consistent
                ui.text(&format!("{}: {}", label, data[0]))
            }
            None => {
                // Values are inconsistent
                let style_token =
                    ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]);
                ui.text(&format!("{}: ", label));
                style_token.pop();
            }
        }
    }

    fn render_mut(
        data: &mut [&mut u32],
        label: &'static str,

        ui: &imgui::Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        let same_or_none_value = get_same_or_none_mut(data);

        let value = same_or_none_value.unwrap_or(0);

        // CAST
        let mut value = value as i32;

        let style_token = if same_or_none_value.is_none() {
            // If values are inconsistent, push a style
            Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        } else {
            None
        };

        let mut changed = false;
        if imgui::InputInt::new(ui, &label, &mut value).build()
            && value >= args.min_value.map(|x| x as i32).unwrap_or(0).max(0)
            && value <= args.max_value.map(|x| x as i32).unwrap_or(i32::MAX)
        {
            {
                for d in data {
                    // CAST
                    let value = value as u32;

                    **d = value;
                    changed = true;
                }
            }
        }

        if let Some(style_token) = style_token {
            style_token.pop();
        }

        changed
    }
}
