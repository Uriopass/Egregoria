use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::input::MouseButton;
use yakui_core::paint::PaintRect;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::Response;
use yakui_widgets::shapes::RoundedRectangle;

/**
A colored box that can contain children.

Responds with [InteractBoxResponse].
 */
#[derive(Debug, Clone)]
pub struct InteractBox {
    pub color: Color,
    pub hover_color: Color,
    pub click_color: Color,
    pub border_radius: f32,
}

impl InteractBox {
    pub fn empty() -> Self {
        Self {
            color: Color::WHITE,
            hover_color: Color::WHITE,
            click_color: Color::WHITE,
            border_radius: 0.0,
        }
    }

    pub fn show(self) -> Response<InteractBoxResponse> {
        yakui_widgets::util::widget::<InteractBoxWidget>(self)
    }

    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<InteractBoxResponse> {
        yakui_widgets::util::widget_children::<InteractBoxWidget, F>(children, self)
    }
}

pub fn interact_box_radius(
    color: Color,
    hover_color: Color,
    click_color: Color,
    border_radius: f32,
    children: impl FnOnce(),
) -> Response<InteractBoxResponse> {
    InteractBox {
        color,
        hover_color,
        click_color,
        border_radius,
    }
    .show_children(children)
}

pub fn interact_box(
    color: Color,
    hover_color: Color,
    click_color: Color,
    children: impl FnOnce(),
) -> Response<InteractBoxResponse> {
    InteractBox {
        color,
        hover_color,
        click_color,
        border_radius: 0.0,
    }
    .show_children(children)
}

#[derive(Debug)]
pub struct InteractBoxWidget {
    props: InteractBox,
    resp: InteractBoxResponse,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct InteractBoxResponse {
    pub hovered: bool,
    pub mousedown: bool,
    pub just_hovered: bool,
    pub just_clicked: bool,
}

impl Widget for InteractBoxWidget {
    type Props<'a> = InteractBox;
    type Response = InteractBoxResponse;

    fn new() -> Self {
        Self {
            props: InteractBox::empty(),
            resp: InteractBoxResponse::default(),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
        let resp = self.resp;
        self.resp.just_hovered = false;
        self.resp.just_clicked = false;
        resp
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let curcolor = if self.resp.mousedown {
            self.props.click_color
        } else if self.resp.hovered {
            self.props.hover_color
        } else {
            self.props.color
        };

        if self.props.border_radius > 0.0 {
            let mut rect = RoundedRectangle::new(layout_node.rect, self.props.border_radius);
            rect.color = curcolor;
            rect.add(ctx.paint);
        } else {
            let mut rect = PaintRect::new(layout_node.rect);
            rect.color = curcolor;
            rect.add(ctx.paint);
        }

        for &child in &node.children {
            ctx.paint(child);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL
    }

    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => {
                self.resp.just_hovered = true;
                self.resp.hovered = true;
                EventResponse::Bubble
            }
            WidgetEvent::MouseLeave => {
                self.resp.hovered = false;
                EventResponse::Bubble
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                ..
            } => {
                if *down && *inside {
                    self.resp.just_clicked = true;
                    self.resp.mousedown = true;
                } else {
                    self.resp.mousedown = false;
                }
                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}
