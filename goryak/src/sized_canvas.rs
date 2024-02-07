use std::cell::Cell;
use std::fmt::Debug;
use yakui_core::geometry::{Color, Constraints, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::widget::{LayoutContext, PaintContext, Widget};
use yakui_core::Response;
use yakui_widgets::util::widget;

type DrawCallback = Box<dyn FnOnce(&mut PaintContext<'_>) + 'static>;

/**
Allows the user to draw arbitrary graphics in a region.

Responds with [SizedCanvasResponse].
 */
pub struct SizedCanvas {
    draw: Cell<Option<DrawCallback>>,
    pub size: Vec2,
    pub bg_color: Option<Color>,
}

impl Debug for SizedCanvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SizedCanvas").finish()
    }
}

impl SizedCanvas {
    pub fn new(
        size: Vec2,
        bg_color: Option<Color>,
        draw: impl FnOnce(&mut PaintContext<'_>) + 'static,
    ) -> Self {
        Self {
            draw: Cell::new(Some(Box::new(draw))),
            size,
            bg_color,
        }
    }

    pub fn show(self) -> Response<SizedCanvasResponse> {
        widget::<SizedCanvasWidget>(self)
    }
}

pub fn sized_canvas(
    size: Vec2,
    bg_color: Color,
    draw: impl FnOnce(&mut PaintContext<'_>) + 'static,
) -> Response<SizedCanvasResponse> {
    SizedCanvas::new(size, Some(bg_color), draw).show()
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
                draw: Cell::new(None),
                size: Default::default(),
                bg_color: None,
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
        if let Some(bg_color) = self.props.bg_color {
            let this_rect = ctx.layout.get(ctx.dom.current()).unwrap().rect;
            let mut p = PaintRect::new(this_rect);
            p.color = bg_color;
            p.add(ctx.paint);
        }

        if let Some(draw) = self.props.draw.take() {
            draw(&mut ctx);
        }

        self.default_paint(ctx);
    }
}
