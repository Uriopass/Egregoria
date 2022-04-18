use super::{
    get_same_or_none, get_same_or_none_mut, imgui, InspectArgsDefault, InspectRenderDefault,
};

impl InspectRenderDefault<f64> for f64 {
    fn render(data: &[&f64], label: &'static str, ui: &imgui::Ui<'_>, _args: &InspectArgsDefault) {
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
                let mut cp = *data[0];
                imgui::Drag::new(&*label).build(ui, &mut cp);
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
        data: &mut [&mut f64],
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
        if imgui::Drag::new(&label).build(ui, &mut value) {
            for d in data {
                **d = value;
                changed = true;
            }
        }

        if let Some(style_token) = style_token {
            style_token.pop();
        }

        changed
    }
}
