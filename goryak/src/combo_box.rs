use crate::{background, button_secondary};
use yakui_core::geometry::{Constraints, Dim2, Vec2};
use yakui_core::{Alignment, CrossAxisAlignment, MainAxisAlignment, MainAxisSize, Pivot};
use yakui_widgets::widgets::{Layer, List, Pad};
use yakui_widgets::{colored_box_container, constrained, pad, reflow, use_state};

pub fn combo_box(selected: &mut usize, items: &[&str], w: f32) -> bool {
    let mut changed = false;

    constrained(Constraints::loose(Vec2::new(w, f32::INFINITY)), || {
        pad(Pad::horizontal(10.0), || {
            let mut l = List::column();
            l.main_axis_alignment = MainAxisAlignment::Center;
            l.main_axis_size = MainAxisSize::Min;
            l.cross_axis_alignment = CrossAxisAlignment::Stretch;
            l.show(|| {
                let open = use_state(|| false);

                if button_secondary(items[*selected].to_string())
                    .show()
                    .clicked
                {
                    open.modify(|x| !x);
                }

                if open.get() {
                    Layer::new().show(|| {
                        reflow(Alignment::BOTTOM_LEFT, Pivot::TOP_LEFT, Dim2::ZERO, || {
                            constrained(Constraints::loose(Vec2::new(w, f32::INFINITY)), || {
                                colored_box_container(background(), || {
                                    Pad::all(3.0).show(|| {
                                        let mut l = List::column();
                                        l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                                        l.item_spacing = 3.0;
                                        l.show(|| {
                                            for (i, item) in items.iter().enumerate() {
                                                if button_secondary(item.to_string()).show().clicked
                                                {
                                                    open.set(false);
                                                    if i != *selected {
                                                        *selected = i;
                                                        changed = true;
                                                    }
                                                }
                                            }
                                        });
                                    });
                                });
                            });
                        });
                    });
                }
            });
        });
    });

    changed
}
