use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::input::MouseButton;
use yakui_core::paint::PaintRect;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::{Response, TextureId};

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
}

impl ImageButton {
    pub fn empty() -> Self {
        Self {
            texture: None,
            size: Vec2::ZERO,
            color: Color::WHITE,
            hover_color: Color::WHITE,
            active_color: Color::WHITE,
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
) -> Response<ImageButtonResponse> {
    ImageButton {
        texture: Some(texture),
        size,
        color,
        hover_color,
        active_color,
    }
    .show()
}

#[derive(Debug)]
pub struct ImageButtonWidget {
    props: ImageButton,
    resp: ImageButtonResponse,
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
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
        let resp = self.resp;
        self.resp.mouse_entered = false;
        self.resp.clicked = false;
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
        input.constrain_min(self.props.size)
    }

    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match event {
            WidgetEvent::MouseEnter => {
                self.resp.mouse_entered = true;
                self.resp.hovering = true;
                EventResponse::Bubble
            }
            WidgetEvent::MouseLeave => {
                self.resp.hovering = false;
                EventResponse::Bubble
            }
            WidgetEvent::MouseButtonChanged {
                button: MouseButton::One,
                down,
                inside,
                ..
            } => {
                if *down && *inside {
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
