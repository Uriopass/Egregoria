/*
use lazy_static::lazy_static;
use std::sync::Mutex;
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;
use yakui_widgets::util::widget_children;

struct TooltipState(Option<TooltipStateInner>);

struct TooltipStateInner {
    tooltip: &'static str,
    position: Vec2,
}

lazy_static! {
    static ref TOOLTIP_STATE: Mutex<TooltipState> = Mutex::new(TooltipState(None));
}

/// Call this at the end of the gui where the "reflow" refers to the root widget
/// so that we can put the tooltip at the right place but the tooltip is necessarily
/// above everything
pub fn render_tooltip() {
    let mut tooltip_state = TOOLTIP_STATE.lock().unwrap();
    if let Some(tooltip_state_inner) = tooltip_state.0 {}
}

/// Positions the children of this widget as close to the mouse as possible while staying
/// within the bounds of the window.
#[derive(Debug)]
struct PositionTooltip {
    position: Vec2,
}

impl PositionTooltip {
    pub fn new(position: Vec2) -> Self {
        Self { position }
    }

    pub fn show<F: FnOnce()>(self, children: F) -> Response<()> {
        widget_children::<PositionTooltipWidget, F>(children, self)
    }
}

#[derive(Debug)]
struct PositionTooltipWidget {
    props: PositionTooltip,
}

impl Widget for PositionTooltipWidget {
    type Props<'a> = PositionTooltip;
    type Response = ();

    fn new() -> Self {
        Self {
            props: PositionTooltip::new(Vec2::ZERO),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = Vec2::ZERO;
        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            size = size.max(child_size);
        }
        let v = ctx.layout.viewport();

        Vec2::ZERO
    }
}
 */
