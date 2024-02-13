use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::widget::{EventContext, Widget};
use yakui_core::Response;
use yakui_widgets::util::widget_children;

pub fn is_hovered(children: impl FnOnce()) -> Response<IsHoveredResponse> {
    widget_children::<IsHoveredWidget, _>(children, ())
}

#[derive(Debug, Copy, Clone, Default)]
pub struct IsHoveredResponse {
    pub hovered: bool,
}

#[derive(Debug)]
pub struct IsHoveredWidget {
    resp: IsHoveredResponse,
}

impl Widget for IsHoveredWidget {
    type Props<'a> = ();
    type Response = IsHoveredResponse;

    fn new() -> Self {
        Self {
            resp: Default::default(),
        }
    }

    fn update(&mut self, _: Self::Props<'_>) -> Self::Response {
        self.resp
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }

    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseEnter => self.resp.hovered = true,
            WidgetEvent::MouseLeave => self.resp.hovered = false,
            _ => {}
        };
        EventResponse::Bubble
    }
}
