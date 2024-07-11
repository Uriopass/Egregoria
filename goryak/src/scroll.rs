use std::cell::Cell;

use yakui_core::event::{EventInterest, EventResponse, WidgetEvent};
use yakui_core::geometry::{Color, Constraints, FlexFit, Rect, Vec2};
use yakui_core::input::MouseButton;
use yakui_core::widget::{EventContext, LayoutContext, PaintContext, Widget};
use yakui_core::Response;
use yakui_widgets::shapes::RoundedRectangle;

#[derive(Debug, PartialEq)]
pub enum VertScrollSize {
    /// The scroll size is a percentage of the parent's size.
    Percent(f32),
    /// The scroll size is exactly what is given (within constraints).
    Exact(f32),
    /// The scroll size is at most what is given (within constraints).
    Fixed(f32),
    /// The scroll size is equal to the parent's size.
    Max,
}

impl VertScrollSize {
    pub fn show<F: FnOnce()>(self, children: F) -> Response<VertScrollResponse> {
        yakui_widgets::util::widget_children::<VertScrollWidget, F>(
            children,
            VertScroll {
                size: self,
                align_bot: false,
            },
        )
    }
}

#[derive(Debug)]
pub struct VertScroll {
    pub size: VertScrollSize,
    pub align_bot: bool,
}

impl VertScroll {
    pub fn show<F: FnOnce()>(self, children: F) -> Response<VertScrollResponse> {
        yakui_widgets::util::widget_children::<VertScrollWidget, F>(children, self)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct VertScrollWidget {
    props: VertScroll,
    scroll_position: Cell<f32>,
    size: Cell<f32>,
    canvas_size: Cell<f32>,
    scrollbar_rect: Cell<Rect>,
    scrollbar_dragging: Cell<Option<f32>>,
    scrollbar_hovered: bool,
}

pub type VertScrollResponse = ();

impl Widget for VertScrollWidget {
    type Props<'a> = VertScroll;
    type Response = VertScrollResponse;

    fn new() -> Self {
        Self {
            props: VertScroll {
                size: VertScrollSize::Max,
                align_bot: false,
            },
            scroll_position: Cell::new(0.0),
            size: Cell::new(0.0),
            canvas_size: Cell::new(0.0),
            scrollbar_rect: Cell::new(Rect::ZERO),
            scrollbar_dragging: Default::default(),
            scrollbar_hovered: false,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flex(&self) -> (u32, FlexFit) {
        match self.props.size {
            VertScrollSize::Max => (1, FlexFit::Tight),
            VertScrollSize::Percent(_) => (1, FlexFit::Loose),
            VertScrollSize::Exact(_) => (0, FlexFit::Tight),
            _ => (0, FlexFit::Loose),
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, mut constraints: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);

        let node = ctx.dom.get_current();
        let mut canvas_size = Vec2::ZERO;

        let main_axis_size = match self.props.size {
            VertScrollSize::Max => constraints.max.y,
            VertScrollSize::Fixed(h) => {
                constraints.max.y = constraints.max.y.min(h);
                constraints.min.y
            }
            VertScrollSize::Exact(h) => {
                constraints.max.y = constraints.max.y.min(h);
                constraints.min.y
            }
            VertScrollSize::Percent(percent) => {
                constraints.max.y *= percent;
                constraints.min.y
            }
        };

        canvas_size.y = main_axis_size;

        let child_constraints = Constraints {
            min: Vec2::new(constraints.min.x, 0.0),
            max: Vec2::new(constraints.max.x, 1000000.0),
        };

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, child_constraints);
            canvas_size = canvas_size.max(child_size);
        }

        let mut size = constraints.constrain(canvas_size);
        if let VertScrollSize::Exact(_) = self.props.size {
            size.y = size.y.max(constraints.max.y);
        }

        self.canvas_size.set(canvas_size.y);
        self.size.set(size.y);

        let max_scroll_position = (canvas_size.y - size.y).max(0.0);
        let scroll_position = self.scroll_position.get().clamp(0.0, max_scroll_position);

        self.scroll_position.set(scroll_position);

        for &child in &node.children {
            let mut off = Vec2::new(0.0, -scroll_position);
            if self.props.align_bot {
                off.y = size.y - canvas_size.y + scroll_position;
            }

            ctx.layout.set_pos(child, off);
        }

        size
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let drawn_rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let node = ctx.dom.get_current();

        for &child in &node.children {
            ctx.paint(child);
        }

        let mut scrollbar_width: f32 = 4.0;
        if self.scrollbar_hovered {
            scrollbar_width = 5.0;
        }
        const SCROLLBAR_PAD_X: f32 = 4.0;
        const SCROLLBAR_PAD_Y: f32 = 2.0;

        if self.canvas_size.get() <= drawn_rect.size().y {
            self.scrollbar_rect.set(Rect::ZERO);
            return;
        }
        let scrollbar_progress =
            self.scroll_position.get() / (self.canvas_size.get() - self.size.get());
        let scroll_bar_height =
            drawn_rect.size().y * (drawn_rect.size().y / self.canvas_size.get());
        let remaining_space = drawn_rect.size().y - scroll_bar_height - SCROLLBAR_PAD_Y;

        let mut pos_y = remaining_space * scrollbar_progress + SCROLLBAR_PAD_Y * 0.5;

        if self.props.align_bot {
            pos_y = drawn_rect.size().y - scroll_bar_height - pos_y;
        }

        let scroll_bar_pos = drawn_rect.pos()
            + Vec2::new(
                drawn_rect.size().x - scrollbar_width * 0.5 - SCROLLBAR_PAD_X,
                pos_y,
            );
        let scroll_bar_rect = Rect::from_pos_size(
            scroll_bar_pos,
            Vec2::new(scrollbar_width, scroll_bar_height),
        );

        self.scrollbar_rect.set(scroll_bar_rect);

        let mut paint_rect = RoundedRectangle::new(scroll_bar_rect, 5.0);

        let mut alpha = 0.5;
        if self.scrollbar_hovered {
            alpha = 0.7;
        }
        if self.scrollbar_dragging.get().is_some() {
            alpha = 1.0;
        }

        paint_rect.color = Color::WHITE.with_alpha(alpha);
        paint_rect.add(ctx.paint);
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseScroll { mut delta } => {
                if self.props.align_bot {
                    delta.y = -delta.y;
                }
                *self.scroll_position.get_mut() += delta.y;
                EventResponse::Sink
            }
            WidgetEvent::MouseMoved(Some(pos)) => {
                if let Some(last_pos) = self.scrollbar_dragging.get_mut() {
                    *self.scroll_position.get_mut() +=
                        (pos.y - *last_pos) * (*self.canvas_size.get_mut() / *self.size.get_mut());
                    *last_pos = pos.y;
                    return EventResponse::Sink;
                } else {
                    self.scrollbar_hovered = self.scrollbar_rect.get_mut().contains_point(pos);
                }
                EventResponse::Bubble
            }
            WidgetEvent::MouseButtonChanged {
                position,
                button: MouseButton::One,
                down,
                ..
            } => {
                if !down {
                    *self.scrollbar_dragging.get_mut() = None;
                    return EventResponse::Bubble;
                }
                if self.scrollbar_rect.get_mut().contains_point(position) {
                    *self.scrollbar_dragging.get_mut() = Some(position.y);
                    return EventResponse::Sink;
                }
                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HorizScrollSize {
    /// The scroll size is a percentage of the parent's size.
    Percent(f32),
    /// The scroll size is exactly what is given (within constraints).
    Exact(f32),
    /// The scroll size is at most what is given (within constraints).
    Fixed(f32),
    /// The scroll size is equal to the parent's size.
    Max,
}

impl HorizScrollSize {
    pub fn show<F: FnOnce()>(self, children: F) -> Response<HorizScrollResponse> {
        yakui_widgets::util::widget_children::<HorizScrollWidget, F>(
            children,
            HorizScroll { size: self },
        )
    }
}

#[derive(Debug)]
pub struct HorizScroll {
    pub size: HorizScrollSize,
}

impl HorizScroll {
    pub fn show<F: FnOnce()>(self, children: F) -> Response<HorizScrollResponse> {
        yakui_widgets::util::widget_children::<HorizScrollWidget, F>(children, self)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct HorizScrollWidget {
    props: HorizScroll,
    scroll_position: Cell<f32>,
    size: Cell<f32>,
    canvas_size: Cell<f32>,
    scrollbar_rect: Cell<Rect>,
    scrollbar_dragging: Cell<Option<f32>>,
    scrollbar_hovered: bool,
}

pub type HorizScrollResponse = ();

impl Widget for HorizScrollWidget {
    type Props<'a> = HorizScroll;
    type Response = HorizScrollResponse;

    fn new() -> Self {
        Self {
            props: HorizScroll {
                size: HorizScrollSize::Max,
            },
            scroll_position: Cell::new(0.0),
            size: Cell::new(0.0),
            canvas_size: Cell::new(0.0),
            scrollbar_rect: Cell::new(Rect::ZERO),
            scrollbar_dragging: Default::default(),
            scrollbar_hovered: false,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn flex(&self) -> (u32, FlexFit) {
        match self.props.size {
            HorizScrollSize::Max => (1, FlexFit::Tight),
            HorizScrollSize::Percent(_) => (1, FlexFit::Loose),
            HorizScrollSize::Exact(_) => (0, FlexFit::Tight),
            _ => (0, FlexFit::Loose),
        }
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, mut constraints: Constraints) -> Vec2 {
        if self.props.size != HorizScrollSize::Max {
            ctx.layout.enable_clipping(ctx.dom);
        }

        let node = ctx.dom.get_current();
        let mut canvas_size = Vec2::ZERO;

        let main_axis_size = match self.props.size {
            HorizScrollSize::Max => constraints.max.x,
            HorizScrollSize::Fixed(h) => {
                constraints.max.x = constraints.max.x.min(h);
                constraints.min.x
            }
            HorizScrollSize::Exact(h) => {
                constraints.max.x = constraints.max.x.min(h);
                constraints.min.x
            }
            HorizScrollSize::Percent(percent) => {
                constraints.max.x *= percent;
                constraints.min.x
            }
        };

        canvas_size.x = main_axis_size;

        let child_constraints = Constraints {
            min: Vec2::new(0.0, constraints.min.y),
            max: Vec2::new(1000000.0, constraints.max.y),
        };

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, child_constraints);
            canvas_size = canvas_size.max(child_size);
        }

        let mut size = constraints.constrain(canvas_size);
        if let HorizScrollSize::Exact(_) = self.props.size {
            size.x = size.x.max(constraints.max.x);
        }

        self.canvas_size.set(canvas_size.x);
        self.size.set(size.x);

        let max_scroll_position = (canvas_size.x - size.x).max(0.0);
        let scroll_position = self.scroll_position.get().clamp(0.0, max_scroll_position);

        self.scroll_position.set(scroll_position);

        for &child in &node.children {
            let off = Vec2::new(-scroll_position, 0.0);
            ctx.layout.set_pos(child, off);
        }

        size
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let drawn_rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
        let node = ctx.dom.get_current();

        for &child in &node.children {
            ctx.paint(child);
        }

        let mut scrollbar_height: f32 = 4.0;
        if self.scrollbar_hovered {
            scrollbar_height = 5.0;
        }
        const SCROLLBAR_PAD_Y: f32 = 4.0;
        const SCROLLBAR_PAD_X: f32 = 2.0;

        if self.canvas_size.get() <= drawn_rect.size().x {
            self.scrollbar_rect.set(Rect::ZERO);
            return;
        }
        let scrollbar_progress =
            self.scroll_position.get() / (self.canvas_size.get() - self.size.get());
        let scroll_bar_width = drawn_rect.size().x * (drawn_rect.size().x / self.canvas_size.get());
        let remaining_space = drawn_rect.size().x - scroll_bar_width - SCROLLBAR_PAD_X;

        let pos_x = remaining_space * scrollbar_progress + SCROLLBAR_PAD_X * 0.5;

        let scroll_bar_pos = drawn_rect.pos()
            + Vec2::new(
                pos_x,
                drawn_rect.size().y - scrollbar_height * 0.5 - SCROLLBAR_PAD_Y,
            );
        let scroll_bar_rect = Rect::from_pos_size(
            scroll_bar_pos,
            Vec2::new(scroll_bar_width, scrollbar_height),
        );

        self.scrollbar_rect.set(scroll_bar_rect);

        let mut paint_rect = RoundedRectangle::new(scroll_bar_rect, 5.0);

        let mut alpha = 0.5;
        if self.scrollbar_hovered {
            alpha = 0.7;
        }
        if self.scrollbar_dragging.get().is_some() {
            alpha = 1.0;
        }

        paint_rect.color = Color::WHITE.with_alpha(alpha);
        paint_rect.add(ctx.paint);
    }

    fn event_interest(&self) -> EventInterest {
        EventInterest::MOUSE_ALL
    }

    fn event(&mut self, _ctx: EventContext<'_>, event: &WidgetEvent) -> EventResponse {
        match *event {
            WidgetEvent::MouseScroll { delta } => {
                *self.scroll_position.get_mut() += delta.y;
                EventResponse::Sink
            }
            WidgetEvent::MouseMoved(Some(pos)) => {
                if let Some(last_pos) = self.scrollbar_dragging.get_mut() {
                    *self.scroll_position.get_mut() +=
                        (pos.x - *last_pos) * (*self.canvas_size.get_mut() / *self.size.get_mut());
                    *last_pos = pos.x;
                    return EventResponse::Sink;
                } else {
                    self.scrollbar_hovered = self.scrollbar_rect.get_mut().contains_point(pos);
                }
                EventResponse::Bubble
            }
            WidgetEvent::MouseButtonChanged {
                position,
                button: MouseButton::One,
                down,
                ..
            } => {
                if !down {
                    *self.scrollbar_dragging.get_mut() = None;
                    return EventResponse::Bubble;
                }
                if self.scrollbar_rect.get_mut().contains_point(position) {
                    *self.scrollbar_dragging.get_mut() = Some(position.x);
                    return EventResponse::Sink;
                }
                EventResponse::Bubble
            }
            _ => EventResponse::Bubble,
        }
    }
}
