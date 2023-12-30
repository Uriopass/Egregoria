use yakui_core::geometry::{Color, Constraints, Rect, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;

pub fn divider(color: Color, height: f32, thickness: f32) -> Response<DividerResponse> {
    Divider::new(color, height, thickness).show()
}

/// A horizontal divider line. Will take up the whole width of the parent.
///
/// The line width is determined by the parent's width after the layout phase.
///
/// Responds with [DividerResponse].
#[derive(Debug)]
#[non_exhaustive]
pub struct Divider {
    /// The color of the divider.
    pub color: Color,
    /// The thickness of the divider.
    pub thickness: f32,
    /// The height of the divider.
    /// How much vertical space it takes up.
    pub height: f32,
    /// The indent of the divider from the left.
    pub indent: f32,
    /// The indent of the divider from the right.
    pub end_indent: f32,
}

impl Divider {
    pub fn new(color: Color, height: f32, thickness: f32) -> Self {
        Self {
            color,
            thickness,
            height,
            indent: 0.0,
            end_indent: 0.0,
        }
    }

    pub fn show(self) -> Response<DividerResponse> {
        yakui_widgets::util::widget::<DividerWidget>(self)
    }
}

#[derive(Debug)]
pub struct DividerWidget {
    props: Divider,
}

pub type DividerResponse = ();

impl Widget for DividerWidget {
    type Props<'a> = Divider;
    type Response = DividerResponse;

    fn new() -> Self {
        Self {
            props: Divider::new(Color::WHITE, 0.0, 0.0),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, _ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        // We say we dont take horizontal space to avoid the divider making
        // the parent wider than it should be.
        Vec2::new(0.0, self.props.height.clamp(input.min.y, input.max.y))
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let id = ctx.dom.current();
        let Some(parent) = ctx.dom.get(id).unwrap().parent else {
            return;
        };
        let line_width = ctx.layout.get(parent).unwrap().rect.size().x;

        let outer_rect = ctx.layout.get(id).unwrap().rect;

        let line_pos = outer_rect.pos()
            + Vec2::new(
                self.props.indent,
                (outer_rect.size().y - self.props.thickness) / 2.0,
            );
        let line_size = Vec2::new(
            line_width - self.props.indent - self.props.end_indent,
            self.props.thickness,
        );

        let mut line_rect = PaintRect::new(Rect::from_pos_size(line_pos, line_size));
        line_rect.color = self.props.color;
        line_rect.add(ctx.paint);
    }
}
