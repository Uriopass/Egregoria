use yakui_core::Response;
use yakui_widgets::widgets::ButtonResponse;

use crate::{button_primary, on_secondary, on_tertiary, secondary, tertiary};

pub fn selectable_label_primary(selected: bool, label: &str) -> Response<ButtonResponse> {
    let mut b = button_primary(label);
    if selected {
        b.style.text.color = on_tertiary();
        b.style.fill = tertiary();
    } else {
        b.style.text.color = on_secondary();
        b.style.fill = secondary();
    }

    b.show()
}
