use super::{
    get_same_or_none, get_same_or_none_mut, imgui, InspectArgsDefault, InspectRenderDefault,
};

impl InspectRenderDefault<&'static str> for &'static str {
    fn render(
        data: &[&&'static str],
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
        data: &mut [&mut &'static str],
        label: &'static str,

        ui: &imgui::Ui<'_>,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.is_empty() {
            // Values are inconsistent
            let style_token = ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.0, 0.0, 1.0]);
            ui.text(&format!("{}: ", label));
            style_token.pop();
            return false;
        }

        match get_same_or_none_mut(data) {
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
        false
    }
}
