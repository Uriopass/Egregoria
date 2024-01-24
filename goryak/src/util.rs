use std::borrow::Cow;
use std::panic::Location;

use yakui_core::geometry::{Color, Constraints, FlexFit, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::{context, Response, WidgetId};
use yakui_widgets::util::widget;
use yakui_widgets::widgets::{Button, Pad, PadResponse, Text};

use crate::{on_primary, on_secondary, primary, secondary, Scrollable, ScrollableResponse};

pub fn scroll_vertical(children: impl FnOnce()) -> Response<ScrollableResponse> {
    Scrollable::vertical().show(children)
}

pub fn checkbox_value(v: &mut bool) {
    *v = yakui_widgets::checkbox(*v).checked;
}

pub fn use_changed<T: Copy + PartialEq + 'static>(v: T, f: impl FnOnce()) {
    let old_v = yakui_widgets::use_state(|| None);
    if old_v.get() != Some(v) {
        old_v.set(Some(v));
        f();
    }
}

pub fn padxy(x: f32, y: f32, children: impl FnOnce()) -> Response<PadResponse> {
    Pad::balanced(x, y).show(children)
}

pub fn pady(y: f32, children: impl FnOnce()) -> Response<PadResponse> {
    Pad::vertical(y).show(children)
}

pub fn padx(x: f32, children: impl FnOnce()) -> Response<PadResponse> {
    Pad::horizontal(x).show(children)
}

pub fn labelc(c: Color, text: impl Into<Cow<'static, str>>) {
    let mut t = Text::label(text.into());
    t.style.color = c;
    t.show();
}

#[derive(Debug)]
pub struct FixedSizeWidget {
    props: Vec2,
}

pub fn fixed_spacer(size: impl Into<Vec2>) -> Response<<FixedSizeWidget as Widget>::Response> {
    widget::<FixedSizeWidget>(size.into())
}

impl Widget for FixedSizeWidget {
    type Props<'a> = Vec2;
    type Response = ();

    fn new() -> Self {
        Self { props: Vec2::ZERO }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flex(&self) -> (u32, FlexFit) {
        (0, FlexFit::Tight)
    }

    fn layout(&self, _ctx: LayoutContext<'_>, _constraints: Constraints) -> Vec2 {
        self.props
    }

    fn paint(&self, _ctx: PaintContext<'_>) {}
}

pub fn widget_inner<T, F, U>(children: F, props: T::Props<'_>) -> U
where
    T: Widget,
    F: FnOnce() -> U,
{
    let dom = context::dom();
    let response = dom.begin_widget::<T>(props);
    let r = children();
    dom.end_widget::<T>(response.id);
    r
}

pub fn button_primary(text: impl Into<String>) -> Button {
    let mut b = Button::styled(text.into());
    b.style.fill = primary();
    b.style.text.color = on_primary();
    b.hover_style.fill = primary().adjust(1.2);
    b.hover_style.text.color = on_primary();
    b.down_style.fill = primary().adjust(1.3);
    b.down_style.text.color = on_primary();
    b
}

pub fn button_secondary(text: impl Into<String>) -> Button {
    let mut b = Button::styled(text.into());
    b.style.fill = secondary();
    b.style.text.color = on_secondary();
    b.hover_style.fill = secondary().adjust(1.2);
    b.hover_style.text.color = on_secondary();
    b.down_style.fill = secondary().adjust(1.3);
    b.down_style.text.color = on_secondary();
    b
}

#[track_caller]
pub fn debug_size<T>(r: Response<T>) -> Response<T> {
    widget::<DebugSize>(DebugSize {
        id: Some(r.id),
        loc: Location::caller(),
    });
    r
}

#[track_caller]
pub fn debug_size_id(id: WidgetId) {
    widget::<DebugSize>(DebugSize {
        id: Some(id),
        loc: Location::caller(),
    });
}

#[derive(Debug)]
struct DebugSize {
    id: Option<WidgetId>,
    loc: &'static Location<'static>,
}

impl Widget for DebugSize {
    type Props<'a> = DebugSize;
    type Response = ();

    fn new() -> Self {
        Self {
            id: None,
            loc: Location::caller(),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        *self = props;
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let layout = ctx.layout;
        let Some(id) = self.id else {
            return;
        };
        let Some(node) = layout.get(id) else {
            eprintln!("{}: not found", self.loc);
            return;
        };
        eprintln!("{}: {:?}", self.loc, node.rect);
    }
}

#[track_caller]
pub fn debug_constraints() {
    yakui_widgets::util::widget::<DebugConstraints>(Location::caller());
}

#[derive(Debug)]
pub struct DebugConstraints {
    props: &'static Location<'static>,
}

impl Widget for DebugConstraints {
    type Props<'a> = &'static Location<'static>;
    type Response = ();

    fn new() -> Self {
        Self {
            props: Location::caller(),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        println!("{}: {:?}", self.props, constraints);
        Widget::default_layout(self, ctx, constraints)
    }
}
