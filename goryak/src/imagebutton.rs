use std::time::Instant;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Dim2, Rect, Vec2};
use yakui_core::input::MouseButton;
use yakui_core::paint::PaintRect;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::{Alignment, Response, TextureId};
use yakui_widgets::{offset, reflow};

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
    pub tooltip: &'static str,
}

impl ImageButton {
    pub fn empty() -> Self {
        Self {
            texture: None,
            size: Vec2::ZERO,
            color: Color::WHITE,
            hover_color: Color::WHITE,
            active_color: Color::WHITE,
            tooltip: "",
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
            tooltip: "",
        }
    }

    pub fn show(self) -> Response<ImageButtonResponse> {
        yakui_widgets::util::widget::<ImageButtonWidget>(self)
    }
}

pub fn image_button(
    texture: TextureId,
    size: Vec2,
    color: Color,
    hover_color: Color,
    active_color: Color,
    tooltip: &'static str,
) -> Response<ImageButtonResponse> {
    ImageButton {
        texture: Some(texture),
        size,
        color,
        hover_color,
        active_color,
        tooltip,
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
                    reflow(Alignment::TOP_LEFT, Dim2::pixels(0.0, 0.0), || {
                        offset(Vec2::new(-10.0, -50.0), || {
                            round_rect(5.0, primary(), || {
                                padxy(5.0, 4.0, || {
                                    textc(on_primary(), self.props.tooltip);
                                });
                            });
                        });
                    });
                }
            }
        }

        resp
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

    fn layout(&self, _ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let _ = self.default_layout(_ctx, input); // tooltip is reflowed

        input.constrain_min(self.props.size)
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
