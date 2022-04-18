use super::{
    get_same_or_none, get_same_or_none_mut, imgui, InspectArgsDefault, InspectRenderDefault,
};

impl InspectRenderDefault<String> for String {
    fn render(
        data: &[&String],
        label: &'static str,
        ui: &imgui::Ui<'_>,
        _args: &InspectArgsDefault,
    ) {
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
        data: &mut [&mut String],
        label: &'static str,

        ui: &imgui::Ui<'_>,
        _args: &InspectArgsDefault,
    ) -> bool {
        let same_or_none_value = get_same_or_none_mut(data);

        let style_token = if same_or_none_value.is_none() {
            // If values are inconsistent, push a style
            Some(ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]))
        } else {
            None
        };

        let mut value = match same_or_none_value {
            Some(v) => v,
            None => "".to_string(), // Some reasonable default
        };

        let mut changed = false;
        if ui.input_text(label, &mut value).build() {
            for d in data {
                **d = value.to_string();
                changed = true;
            }
        }

        if let Some(style_token) = style_token {
            style_token.pop();
        }

        changed
    }
}
