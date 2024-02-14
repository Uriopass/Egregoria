use crate::DEFAULT_FONT_SIZE;
use std::borrow::Cow;
use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::Response;
use yakui_widgets::constrained;
use yakui_widgets::font::FontName;
use yakui_widgets::widgets::{Text, TextBox, TextResponse};

pub fn text<S: Into<Cow<'static, str>>>(text: S) -> Response<TextResponse> {
    Text::new(DEFAULT_FONT_SIZE, text.into()).show()
}

pub fn monospace<S: Into<Cow<'static, str>>>(col: Color, text: S) -> Response<TextResponse> {
    let mut t = Text::new(DEFAULT_FONT_SIZE, text.into());
    t.style.font = FontName::new("monospace");
    t.style.color = col;
    t.show()
}

pub fn text_edit(width: f32, x: &mut String, placeholder: &str) -> bool {
    let mut activated = false;
    constrained(
        Constraints {
            min: Vec2::new(width, 20.0),
            max: Vec2::new(f32::INFINITY, f32::INFINITY),
        },
        || {
            let mut text = TextBox::new(x.clone());
            text.placeholder = placeholder.to_string();
            text.fill = Some(Color::rgba(0, 0, 0, 50));
            let resp = text.show().into_inner();
            if let Some(changed) = resp.text {
                *x = changed;
            }
            activated = resp.activated;
        },
    );
    activated
}
