use yakui_core::geometry::Vec2;
use yakui_core::{CrossAxisAlignment, MainAxisSize};
use yakui_widgets::widgets::{List, Pad};
use yakui_widgets::{draggable, pad, use_state};

use crate::roundrect::RoundRect;
use crate::{on_primary, outline, secondary, textc};

pub trait Draggable: Copy {
    const DEFAULT_STEP: f64;
    const DEFAULT_MIN: f64;
    const DEFAULT_MAX: f64;

    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
    fn default_step() -> f64;
}

macro_rules! impl_slidable {
    ($($t:ty; $step:expr),*) => {
        $(
            impl Draggable for $t {
                const DEFAULT_STEP: f64 = $step;
                const DEFAULT_MIN: f64 = <$t>::MIN as f64;
                const DEFAULT_MAX: f64 = <$t>::MAX as f64;

                fn to_f64(self) -> f64 {
                    self as f64
                }

                fn from_f64(v: f64) -> Self {
                    v as Self
                }

                fn default_step() -> f64 {
                    $step
                }

            }
        )*
    };
}

impl_slidable!(u8; 1.0,
               i8; 1.0,
               u16; 1.0,
               i16; 1.0,
               u32; 1.0,
               i32; 1.0, 
               u64; 1.0,
               i64; 1.0,
               usize; 1.0,
               isize; 1.0,
               f32; 0.01, 
               f64; 0.01);

pub struct DragValue {
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
}

impl DragValue {
    pub fn min(mut self, v: f64) -> Self {
        self.min = Some(v);
        self
    }

    pub fn max(mut self, v: f64) -> Self {
        self.max = Some(v);
        self
    }
    pub fn minmax(mut self, r: std::ops::Range<f64>) -> Self {
        self.min = Some(r.start);
        self.max = Some(r.end);
        self
    }

    pub fn step(mut self, v: f64) -> Self {
        self.step = Some(v);
        self
    }

    /// Returns true if the value was changed.
    pub fn show<T: Draggable>(self, value: &mut T) -> bool {
        let mut changed = false;
        let step = self.step.unwrap_or(T::default_step());

        let mut l = List::column();
        l.cross_axis_alignment = CrossAxisAlignment::Center;
        l.main_axis_size = MainAxisSize::Min;
        l.show(|| {
            let dragged = draggable_delta(|| {
                RoundRect::new(2.0)
                    .outline(outline(), 2.0)
                    .color(secondary())
                    .show_children(|| {
                        pad(Pad::horizontal(10.0), || {
                            let v = T::to_f64(*value);
                            let text = if step < 0.01 {
                                format!("{:.3}", v)
                            } else if step < 0.1 {
                                format!("{:.2}", v)
                            } else if step < 1.0 {
                                format!("{:.1}", v)
                            } else {
                                format!("{:.0}", v)
                            };
                            textc(on_primary(), text);
                        });
                    });
            });
            if let Some(dragged) = dragged {
                let oldv = T::to_f64(*value);
                let mut newv = oldv + dragged.x as f64 * step;

                newv = (newv / step).round() * step;

                *value = T::from_f64(newv.clamp(
                    self.min.unwrap_or(T::DEFAULT_MIN),
                    self.max.unwrap_or(T::DEFAULT_MAX),
                ));

                changed = true;
            }
        });

        changed
    }
}

fn draggable_delta(children: impl FnOnce()) -> Option<Vec2> {
    let last_val_state = use_state(|| None);
    let Some(mut d) = draggable(children).dragging else {
        last_val_state.set(None);
        return None;
    };

    let last_val = last_val_state.get().unwrap_or(d.current);
    let mut delta = (d.current - last_val) / 10.0;
    if delta.x.abs() < 1.0 {
        d.current.x = last_val.x;
        delta.x = 0.0;
    }
    if delta.y.abs() < 1.0 {
        d.current.y = last_val.y;
        delta.y = 0.0;
    }
    last_val_state.set(Some(d.current));
    Some(delta)
}

pub fn dragvalue() -> DragValue {
    DragValue {
        min: None,
        max: None,
        step: None,
    }
}
