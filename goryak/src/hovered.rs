use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::widget::{EventContext, Widget};
use yakui_widgets::util::widget;

pub fn is_hovered() -> bool {
    *widget::<IsHovered>(())
}

#[derive(Debug)]
pub struct IsHovered {
    hovered: bool,
}

impl Widget for IsHovered {
    type Props<'a> = ();
    type Response = bool;

    fn new() -> Self {
        Self { hovered: false }
    }

    fn update(&mut self, _: Self::Props<'_>) -> Self::Response {
        self.hovered
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }

    fn event(&mut self, _: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseEnter => self.hovered = true,
            WidgetEvent::MouseLeave => self.hovered = false,
            _ => {}
        };
        EventResponse::Bubble
    }
}
