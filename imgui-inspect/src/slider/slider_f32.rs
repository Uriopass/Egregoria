use super::*;

impl InspectRenderSlider<f32> for f32 {
    fn render(data: &[&Self], label: &'static str, ui: &imgui::Ui, _args: &InspectArgsSlider) {
        if data.is_empty() {
            ui.text(&imgui::im_str!("{}: None", label));
            return;
        }

        ui.text(&imgui::im_str!("{}: {}", label, data[0]));
    }

    fn render_mut(
        data: &mut [&mut Self],
        label: &'static str,

        ui: &imgui::Ui,
        args: &InspectArgsSlider,
    ) -> bool {
        let same_or_none_value = get_same_or_none_mut(data);

        let mut value = same_or_none_value.unwrap_or(0.0);

        let style_token = if same_or_none_value.is_none() {
            // If values are inconsistent, push a style
            Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        } else {
            None
        };

        let mut min = -100.0;
        let mut max = 100.0;
        if let Some(min_value) = args.min_value {
            min = min_value;
        }

        if let Some(max_value) = args.max_value {
            max = max_value;
        }

        let mut changed = false;
        if imgui::Slider::new(&imgui::im_str!("{}", label))
            .range(min..=max)
            .build(ui, &mut value)
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
