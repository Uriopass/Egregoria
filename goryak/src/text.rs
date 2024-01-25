use crate::DEFAULT_FONT_SIZE;
use std::borrow::Cow;
use yakui_core::geometry::Color;
use yakui_core::Response;
use yakui_widgets::font::FontName;
use yakui_widgets::widgets::{Text, TextResponse};

pub fn text<S: Into<Cow<'static, str>>>(text: S) -> Response<TextResponse> {
    Text::new(DEFAULT_FONT_SIZE, text.into()).show()
}

pub fn monospace<S: Into<Cow<'static, str>>>(col: Color, text: S) -> Response<TextResponse> {
    let mut t = Text::new(DEFAULT_FONT_SIZE, text.into());
    t.style.font = FontName::new("monospace");
    t.style.color = col;
    t.show()
}
