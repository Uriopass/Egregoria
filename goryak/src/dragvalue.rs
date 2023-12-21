use crate::stretch_width;
use yakui_widgets::pad;
use yakui_widgets::widgets::{Pad, Slider};

pub trait Draggable: Copy {
    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
}

macro_rules! impl_slidable {
    ($($t:ty),*) => {
        $(
            impl Draggable for $t {
                fn to_f64(self) -> f64 {
                    self as f64
                }

                fn from_f64(v: f64) -> Self {
                    v as Self
                }
            }
        )*
    };
}

impl_slidable!(i32, u32, i64, u64, f32, f64);

pub fn drag_value<T: Draggable>(amount: &mut T) {
    stretch_width(|| {
        pad(Pad::horizontal(10.0), || {
            let mut slider = Slider::new((*amount).to_f64(), 1.0, 10.0);
            slider.step = Some(1.0);
            if let Some(v) = slider.show().value {
                *amount = Draggable::from_f64(v);
            }
        });
    });
}
