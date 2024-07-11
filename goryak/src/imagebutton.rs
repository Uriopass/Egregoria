use std::borrow::Cow;
use std::time::Instant;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::input::MouseButton;
use yakui_core::paint::PaintRect;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

use crate::{on_primary, padxy, primary, round_rect, textc};

/**
A button based on an image

Responds with [ImageButtonResponse].
 */
#[derive(Debug, Clone)]
pub struct ImageButton {
    pub texture: Option<TextureId>,
    pub size: Vec2,
    pub color: Color,
    pub hover_color: Color,
    pub active_color: Color,
    pub tooltip: Cow<'static, str>,
}

impl ImageButton {
    pub fn empty() -> Self {
        Self {
            texture: None,
            size: Vec2::ZERO,
            color: Color::WHITE,
            hover_color: Color::WHITE,
            active_color: Color::WHITE,
            tooltip: Cow::Borrowed(""),
        }
    }

    pub fn new(
        texture: TextureId,
        size: Vec2,
        color: Color,
        hover_color: Color,
        active_color: Color,
    ) -> Self {
        Self {
            texture: Some(texture),
            size,
            color,
            hover_color,
            active_color,
            tooltip: Cow::Borrowed(""),
        }
    }

    pub fn show(self) -> Response<ImageButtonResponse> {
        yakui_widgets::util::widget::<ImageButtonWidget>(self)
    }
}

pub fn primary_image_button(
    texture: TextureId,
    size: Vec2,
    enabled: bool,
    tooltip: impl Into<Cow<'static, str>>,
) -> Response<ImageButtonResponse> {
    let (default_col, hover_col) = if enabled {
        let c = primary().lerp(&Color::WHITE, 0.3);
        (c, c.with_alpha(0.7))
    } else {
        (Color::WHITE.with_alpha(0.3), Color::WHITE.with_alpha(0.5))
    };
    image_button(texture, size, default_col, hover_col, primary(), tooltip)
}

pub fn image_button(
    texture: TextureId,
    size: Vec2,
    color: Color,
    hover_color: Color,
    active_color: Color,
    tooltip: impl Into<Cow<'static, str>>,
) -> Response<ImageButtonResponse> {
    ImageButton {
        texture: Some(texture),
        size,
        color,
        hover_color,
        active_color,
        tooltip: tooltip.into(),
    }
    .show()
}

#[derive(Debug)]
pub struct ImageButtonWidget {
    props: ImageButton,
    resp: ImageButtonResponse,
    stopped_moving: Option<Instant>,
    show_tooltip: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ImageButtonResponse {
    pub hovering: bool,
    pub mouse_down: bool,
    pub mouse_entered: bool,
    pub clicked: bool,
}

impl Widget for ImageButtonWidget {
    type Props<'a> = ImageButton;
    type Response = ImageButtonResponse;

    fn new() -> Self {
        Self {
            props: ImageButton::empty(),
            resp: ImageButtonResponse::default(),
            stopped_moving: None,
            show_tooltip: false,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
        let resp = self.resp;
        self.resp.mouse_entered = false;
        self.resp.clicked = false;

        if !self.props.tooltip.is_empty() {
            if let Some(i) = self.stopped_moving {
                if i.elapsed().as_millis() > 500 {
                    self.show_tooltip = true;
                }
                if self.show_tooltip {
                    round_rect(5.0, primary(), || {
                        padxy(5.0, 4.0, || {
                            textc(on_primary(), self.props.tooltip.clone());
                        });
                    });
                }
            }
        }

        resp
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        if let Some(tooltip) = ctx.dom.get_current().children.first() {
            let size = ctx.calculate_layout(*tooltip, Constraints::none());
            ctx.layout.set_pos(
                *tooltip,
                Vec2::new((self.props.size.x - size.x) / 2.0, -size.y - 10.0),
            );
        }

        input.constrain_min(self.props.size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let curcolor = if self.resp.mouse_down {
            self.props.active_color
        } else if self.resp.hovering {
            self.props.hover_color
        } else {
            self.props.color
        };

        let mut rect = PaintRect::new(layout_node.rect);
        rect.color = curcolor;
        if let Some(tex) = self.props.texture {
            rect.texture = Some((tex, Rect::ONE));
        }
        rect.add(ctx.paint);

        for &child in &node.children {
            ctx.paint(child);
        }
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL
    }

    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseMoved(Some(_)) => {
                if self.resp.hovering {
                    self.stopped_moving = Some(Instant::now());
                }
                self.show_tooltip = false;
                EventResponse::Bubble
            }
            WidgetEvent::MouseEnter => {
                self.resp.mouse_entered = true;
                self.resp.hovering = true;
                EventResponse::Bubble
            }
            WidgetEvent::MouseLeave => {
                self.resp.hovering = false;
                self.show_tooltip = false;
                self.stopped_moving = None;
                EventResponse::Bubble
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                ..
            } => {
                if down && inside {
                    self.resp.clicked = true;
                    self.resp.mouse_down = true;
                } else {
                    self.resp.mouse_down = false;
                }
                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}
