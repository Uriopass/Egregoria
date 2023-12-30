use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;
use yakui_widgets::shapes;
use yakui_widgets::util::{widget, widget_children};

/**
A colored box with rounded corners that can contain children.

Responds with [RoundRectResponse].
 */
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RoundRect {
    pub radius: f32,
    pub color: Color,
    pub min_size: Vec2,
    pub outline: Color,
    pub outline_thickness: f32,
}

impl RoundRect {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            color: Color::WHITE,
            min_size: Vec2::ZERO,
            outline: Color::BLACK,
            outline_thickness: 0.0,
        }
    }

    pub fn outline(mut self, color: Color, thickness: f32) -> Self {
        self.outline = color;
        self.outline_thickness = thickness;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn min_size(mut self, size: Vec2) -> Self {
        self.min_size = size;
        self
    }

    pub fn show(self) -> Response<RoundRectResponse> {
        widget::<RoundRectWidget>(self)
    }

    pub fn show_children<F: FnOnce()>(self, children: F) -> Response<RoundRectResponse> {
        widget_children::<RoundRectWidget, F>(children, self)
    }
}

pub fn round_rect(
    radius: f32,
    color: Color,
    children: impl FnOnce(),
) -> Response<RoundRectResponse> {
    RoundRect::new(radius).color(color).show_children(children)
}

#[derive(Debug)]
pub struct RoundRectWidget {
    props: RoundRect,
}

pub type RoundRectResponse = ();

impl Widget for RoundRectWidget {
    type Props<'a> = RoundRect;
    type Response = RoundRectResponse;

    fn new() -> Self {
        Self {
            props: RoundRect::new(0.0),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();
        let mut size = self.props.min_size;

        for &child in &node.children {
            let child_size = ctx.calculate_layout(child, input);
            size = size.max(child_size);
        }

        input.constrain_min(size)
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        let node = ctx.dom.get_current();
        let layout_node = ctx.layout.get(ctx.dom.current()).unwrap();

        let thickness = self.props.outline_thickness;

        if thickness > 0.0 {
            let mut outer_rect = shapes::RoundedRectangle::new(layout_node.rect, self.props.radius);
            outer_rect.color = self.props.outline;
            outer_rect.add(ctx.paint);
        }

        let mut inner_rect = layout_node.rect;
        inner_rect.set_size(inner_rect.size() - Vec2::splat(thickness * 2.0));
        inner_rect.set_pos(inner_rect.pos() + Vec2::splat(thickness));

        if self.props.radius > 0.0 {
            let mut inner_rect =
                shapes::RoundedRectangle::new(inner_rect, self.props.radius - thickness);
            inner_rect.color = self.props.color;
            inner_rect.add(ctx.paint);
        } else {
            let mut inner_rect = PaintRect::new(inner_rect);
            inner_rect.color = self.props.color;
            inner_rect.add(ctx.paint);
        }

        for &child in &node.children {
            ctx.paint(child);
        }
    }
}
