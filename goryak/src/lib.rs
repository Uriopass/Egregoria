mod combo_box;
mod count_grid;
mod decoration;
mod dragvalue;
mod hovered;
mod layout;

pub use combo_box::*;
pub use count_grid::*;
pub use decoration::*;
pub use dragvalue::*;
pub use hovered::*;
pub use layout::*;

pub fn checkbox_value(v: &mut bool) {
    *v = yakui_widgets::checkbox(*v).checked;
}

pub fn use_changed<T: Copy + PartialEq + 'static>(v: T, f: impl FnOnce()) {
    let old_v = yakui_widgets::use_state(|| None);
    if old_v.get() != Some(v) {
        old_v.set(Some(v));
        f();
    }
}
