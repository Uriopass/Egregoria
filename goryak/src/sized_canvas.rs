use std::fmt::Debug;
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;
use yakui_widgets::util::widget;

type DrawCallback = Box<dyn Fn(&mut PaintContext<'_>) + 'static>;

/**
Allows the user to draw arbitrary graphics in a region.

Responds with [SizedCanvasResponse].
 */
pub struct SizedCanvas {
    draw: Option<DrawCallback>,
    pub size: Vec2,
}

impl Debug for SizedCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SizedCanvas").finish()
    }
}

impl SizedCanvas {
    pub fn new(size: Vec2, draw: impl Fn(&mut PaintContext<'_>) + 'static) -> Self {
        Self {
            draw: Some(Box::new(draw)),
            size,
        }
    }

    pub fn show(self) -> Response<SizedCanvasResponse> {
        widget::<SizedCanvasWidget>(self)
    }
}

pub fn sized_canvas(
    size: Vec2,
    draw: impl Fn(&mut PaintContext<'_>) + 'static,
) -> Response<SizedCanvasResponse> {
    SizedCanvas::new(size, draw).show()
}

#[derive(Debug)]
pub struct SizedCanvasWidget {
    props: SizedCanvas,
}

pub type SizedCanvasResponse = ();

impl Widget for SizedCanvasWidget {
    type Props<'a> = SizedCanvas;
    type Response = SizedCanvasResponse;

    fn new() -> Self {
        Self {
            props: SizedCanvas {
                draw: None,
                size: Default::default(),
            },
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, ctx: LayoutContext<'_>, _: Constraints) -> Vec2 {
        ctx.layout.enable_clipping(ctx.dom);
        self.props.size
    }

    fn paint(&self, mut ctx: PaintContext<'_>) {
        if let Some(draw) = &self.props.draw {
            draw(&mut ctx);
        }

        self.default_paint(ctx);
    }
}
