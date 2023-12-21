use yakui_core::{CrossAxisAlignment, MainAxisAlignment, MainAxisSize};
use yakui_widgets::widgets::List;

pub fn stretch_width(children: impl FnOnce()) {
    let mut l = List::column();
    l.cross_axis_alignment = CrossAxisAlignment::Stretch;
    l.main_axis_size = MainAxisSize::Min;
    l.show(children);
}

pub fn center_width(children: impl FnOnce()) {
    stretch_width(|| {
        let mut r = List::row();
        r.main_axis_alignment = MainAxisAlignment::Center;
        r.show(children);
    });
}
