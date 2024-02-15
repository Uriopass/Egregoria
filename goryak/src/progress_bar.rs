use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;
use yakui_widgets::shapes::RoundedRectangle;
use yakui_widgets::util::{widget, widget_children};

/// Shows a progress bar.
/// Children will be centered inside the progress bar without other layouting.
#[derive(Debug)]
pub struct ProgressBar {
    /// The value of the progress bar, between 0.0 and 1.0
    pub value: f32,
    pub size: Vec2,
    pub color: Color,
}

impl ProgressBar {
    pub fn show(self) -> Response<ProgressBarResponse> {
        widget::<ProgressBarWidget>(self)
    }

    pub fn show_children(self, children: impl FnOnce()) -> Response<ProgressBarResponse> {
        widget_children::<ProgressBarWidget, _>(children, self)
    }
}

#[derive(Debug)]
pub struct ProgressBarWidget {
    props: ProgressBar,
}

pub type ProgressBarResponse = ();

impl Widget for ProgressBarWidget {
    type Props<'a> = ProgressBar;
    type Response = ();

    fn new() -> Self {
        Self {
            props: ProgressBar {
                value: 0.0,
                size: Vec2::ZERO,
                color: Color::CLEAR,
            },
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, constraints: Constraints) -> Vec2 {
        let size = constraints.constrain(self.props.size);

        for child in &ctx.dom.get_current().children {
            let child_size = ctx.calculate_layout(*child, Constraints::loose(size));
            let offset = (size - child_size) * 0.5;
            ctx.layout.set_pos(*child, offset);
        }

        size
    }

    fn paint(&self, ctx: PaintContext<'_>) {
        let rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;

        let progress = rect.size().x * self.props.value;
        let mut progress_rect = rect;
        progress_rect.set_size(Vec2::new(progress, progress_rect.size().y));

        let mut bg = RoundedRectangle::new(rect, 4.0);
        bg.color = Color::rgba(0, 0, 0, 50);
        bg.add(ctx.paint);

        let mut fg = RoundedRectangle::new(progress_rect, 4.0);
        fg.color = self.props.color;
        fg.add(ctx.paint);

        self.default_paint(ctx);
    }
}
