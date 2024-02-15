use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::{Alignment, Response};

pub fn pivot(align: Alignment, children: impl FnOnce()) -> Response<()> {
    yakui_widgets::util::widget_children::<PivotWidget, _>(children, align)
}

#[derive(Debug)]
pub struct PivotWidget {
    props: Alignment,
}

impl Widget for PivotWidget {
    type Props<'a> = Alignment;
    type Response = ();

    fn new() -> Self {
        Self {
            props: Alignment::TOP_LEFT,
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();

        let mut size = Vec2::ZERO;
        for &child in &node.children {
            size = size.max(ctx.calculate_layout(child, constraints));
        }

        let pivot_offset = -size * self.props.as_vec2();
        for &child in &node.children {
            ctx.layout.set_pos(child, pivot_offset);
        }

        size
    }
}
