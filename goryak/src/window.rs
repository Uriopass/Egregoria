use std::borrow::Cow;
use std::cell::RefCell;

use yakui_core::geometry::{Dim2, Vec2};
use yakui_core::{Alignment, MainAxisSize};
use yakui_widgets::widgets::{List, Pad};
use yakui_widgets::{draggable, offset, reflow, row, use_state};

use crate::{blur_bg, divider, on_secondary_container, outline, secondary_container, textc};

thread_local! {
    /// Remember which windows were drawn. That what we can put them at the bottom of the widget tree to be drawn on top of the rest.
    /// We can also have a way to remember which windows are active, so they can be on top too.
    static WINDOWS: RefCell<Vec<Window>> = Default::default();
}

pub struct Window {
    pub title: &'static str,
    pub pad: Pad,
}

impl Window {
    pub fn show(self, children: impl FnOnce()) {
        reflow(Alignment::TOP_LEFT, Dim2::ZERO, || {
            let off = use_state(|| Vec2::new(300.0, 300.0));

            offset(off.get(), || {
                let r = draggable(|| {
                    blur_bg(secondary_container().with_alpha(0.7), 5.0, || {
                        self.pad.show(|| {
                            let mut l = List::column();
                            l.main_axis_size = MainAxisSize::Min;
                            l.show(|| {
                                row(|| {
                                    textc(on_secondary_container(), Cow::Borrowed(self.title));
                                });
                                divider(outline(), 10.0, 1.0);
                                children();
                            });
                        });
                    });
                });
                if let Some(v) = r.dragging {
                    off.set(v.current);
                }
            });
        });
    }
}
