use yakui_core::geometry::{Color, Vec2};
use yakui_core::paint::PaintRect;
use yakui_core::WidgetId;
use yakui_widgets::canvas;

pub fn horiz_line(width_id: WidgetId) {
    canvas(move |ctx| {
        let w = ctx.layout.get(width_id).unwrap().rect.size().x;
        let layout = ctx.layout.get(ctx.dom.current()).unwrap();

        let mut r = layout.rect;
        r.set_size(Vec2::new(w, 3.0));
        r.set_pos(Vec2::new(r.pos().x, r.pos().y - 3.0 / 2.0));
        let mut pr = PaintRect::new(r);
        pr.color = Color::GRAY.adjust(0.8);
        pr.add(ctx.paint);
    });
}
