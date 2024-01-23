use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::Response;
use yakui_widgets::util::widget_children;

/**
A box that forces a constraint of viewport size onto its child.

Responds with [ConstrainedViewportResponse].
 */
pub fn constrained_viewport<F: FnOnce()>(children: F) -> Response<ConstrainedViewportResponse> {
    widget_children::<ConstrainedViewportWidget, F>(children, ())
}

#[derive(Debug)]
pub struct ConstrainedViewportWidget;

pub type ConstrainedViewportResponse = ();

impl Widget for ConstrainedViewportWidget {
    type Props<'a> = ();
    type Response = ConstrainedViewportResponse;

    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, _props: Self::Props<'_>) -> Self::Response {}

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = Vec2::ZERO;

        let viewport = ctx.layout.viewport();

        let constraints = Constraints {
            min: input.min,
            max: Vec2::min(input.max, viewport.size()),
        };

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, constraints);
            size = size.max(child_size);
        }

        constraints.constrain(size)
    }
}
