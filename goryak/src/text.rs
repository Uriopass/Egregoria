use crate::DEFAULT_FONT_SIZE;
use std::borrow::Cow;
use yakui_core::Response;
use yakui_widgets::widgets::{Text, TextResponse};

pub fn text<S: Into<Cow<'static, str>>>(text: S) -> Response<TextResponse> {
    Text::new(DEFAULT_FONT_SIZE, text.into()).show()
}
