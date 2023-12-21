use yakui_core::geometry::{Color, Constraints, Dim2, Vec2};
use yakui_core::{Alignment, CrossAxisAlignment, MainAxisAlignment, MainAxisSize};
use yakui_widgets::widgets::{Layer, List, Pad};
use yakui_widgets::{button, colored_box_container, constrained, pad, reflow, use_state};

pub fn combo_box(selected: &mut usize, items: &[&str], w: f32) -> bool {
    let mut changed = false;

    pad(Pad::horizontal(10.0), || {
        let mut l = List::column();
        l.main_axis_alignment = MainAxisAlignment::Center;
        l.main_axis_size = MainAxisSize::Min;
        l.cross_axis_alignment = CrossAxisAlignment::Stretch;
        l.show(|| {
            let open = use_state(|| false);

            if button(items[*selected].to_string()).clicked {
                open.modify(|x| !x);
            }

            if open.get() {
                Layer::new().show(|| {
                    reflow(Alignment::BOTTOM_LEFT, Dim2::ZERO, || {
                        constrained(Constraints::loose(Vec2::new(w, f32::INFINITY)), || {
                            colored_box_container(Color::GRAY.adjust(0.8), || {
                                Pad::all(3.0).show(|| {
                                    let mut l = List::column();
                                    l.cross_axis_alignment = CrossAxisAlignment::Stretch;
                                    l.item_spacing = 3.0;
                                    l.show(|| {
                                        for (i, item) in items.iter().enumerate() {
                                            if i == *selected {
                                                continue;
                                            }

                                            if button(item.to_string()).clicked {
                                                *selected = i;
                                                open.set(false);
                                                changed = true;
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

    changed
}
