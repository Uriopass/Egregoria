use crate::primary;
use std::borrow::Cow;
use yakui_core::geometry::Color;
use yakui_widgets::widgets::Button;

pub fn primary_link(text: impl Into<Cow<'static, str>>) -> bool {
    let mut b = Button::unstyled(text);
    b.style.fill = Color::CLEAR;
    b.style.text.color = primary();

    b.hover_style.fill = Color::CLEAR;
    b.style.text.color = primary();

    b.down_style.fill = Color::CLEAR;
    b.style.text.color = primary();
    let resp = b.show();

    resp.clicked
}
