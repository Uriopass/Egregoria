use std::cell::Cell;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Constraints, FlexFit, Rect, Vec2};
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::{MainAxisSize, Response};
use yakui_widgets::shapes::RoundedRectangle;

#[derive(Debug)]
#[non_exhaustive]
pub struct Scrollable {
    pub direction: Option<ScrollDirection>,
    pub main_axis_size: MainAxisSize,
}

impl Scrollable {
    pub fn none() -> Self {
        Scrollable {
            direction: None,
            main_axis_size: MainAxisSize::Min,
        }
    }

    pub fn vertical() -> Self {
        Scrollable {
            direction: Some(ScrollDirection::Y),
            main_axis_size: MainAxisSize::Min,
        }
    }

    pub fn main_axis_size(mut self, main_axis_size: MainAxisSize) -> Self {
        self.main_axis_size = main_axis_size;
        self
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<ScrollableResponse> {
        yakui_widgets::util::widget_children::<ScrollableWidget, F>(children, self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Y,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ScrollableWidget {
    props: Scrollable,
    scroll_position: Cell<Vec2>,
    size: Cell<Vec2>,
    canvas_size: Cell<Vec2>,
}

pub type ScrollableResponse = ();

impl Widget for ScrollableWidget {
    type Props<'a> = Scrollable;
    type Response = ScrollableResponse;

    fn new() -> Self {
        Self {
            props: Scrollable::none(),
            scroll_position: Cell::new(Vec2::ZERO),
            size: Cell::new(Vec2::ZERO),
            canvas_size: Cell::new(Vec2::ZERO),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flex(&self) -> (u32, FlexFit) {
        match self.props.main_axis_size {
            MainAxisSize::Max => (1, FlexFit::Tight),
            MainAxisSize::Min => (0, FlexFit::Loose),
            _ => unimplemented!(),
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);

        let node = ctx.dom.get_current();
        let mut canvas_size = Vec2::ZERO;

        let main_axis_size = match self.props.main_axis_size {
            MainAxisSize::Max => constraints.max.y,
            MainAxisSize::Min => constraints.min.y,
            _ => unimplemented!(),
        };

        canvas_size.y = canvas_size.y.max(main_axis_size);

        let child_constraints = match self.props.direction {
            None => constraints,
            Some(ScrollDirection::Y) => Constraints {
                min: Vec2::new(constraints.min.x, 0.0),
                max: Vec2::new(constraints.max.x, f32::INFINITY),
            },
        };

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, child_constraints);
            canvas_size = canvas_size.max(child_size);
        }

        let size = constraints.constrain(canvas_size);

        self.canvas_size.set(canvas_size);
        self.size.set(size);

        let max_scroll_position = (canvas_size - size).max(Vec2::ZERO);
        let mut scroll_position = self
            .scroll_position
            .get()
            .min(max_scroll_position)
            .max(Vec2::ZERO);

        match self.props.direction {
            None => scroll_position = Vec2::ZERO,
            Some(ScrollDirection::Y) => scroll_position.x = 0.0,
        }

        self.scroll_position.set(scroll_position);

        for &child in &node.children {
            ctx.layout.set_pos(child, -scroll_position);
        }

        size
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let drawn_rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let node = ctx.dom.get_current();

        for &child in &node.children {
            ctx.paint(child);
        }

        const SCROLLBAR_WIDTH: f32 = 4.0;
        const SCROLLBAR_PAD_X: f32 = 2.0;
        const SCROLLBAR_PAD_Y: f32 = 2.0;

        if self.canvas_size.get().y <= drawn_rect.size().y {
            return;
        }
        let scrollbar_progress =
            self.scroll_position.get().y / (self.canvas_size.get().y - self.size.get().y);
        let scroll_bar_height =
            drawn_rect.size().y * (drawn_rect.size().y / self.canvas_size.get().y);
        let remaining_space = drawn_rect.size().y - scroll_bar_height - SCROLLBAR_PAD_Y;

        let scroll_bar_pos = drawn_rect.pos()
            + Vec2::new(
                drawn_rect.size().x - SCROLLBAR_WIDTH - SCROLLBAR_PAD_X,
                remaining_space * scrollbar_progress + SCROLLBAR_PAD_Y * 0.5,
            );
        let scroll_bar_rect = Rect::from_pos_size(
            scroll_bar_pos,
            Vec2::new(SCROLLBAR_WIDTH, scroll_bar_height),
        );

        RoundedRectangle::new(scroll_bar_rect, 5.0).add(ctx.paint);
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_INSIDE
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseScroll { delta } => {
                let pos = self.scroll_position.get();
                self.scroll_position.set(pos + delta);
                EventResponse::Sink
            }
            _ => EventResponse::Bubble,
        }
    }
}
