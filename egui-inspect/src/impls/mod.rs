mod bool;
mod btrees;
mod f32;
mod f64;
pub mod geometry;
mod i32;
mod i64;
mod option;
mod str;
mod string;
mod tuple;
mod u16;
mod u32;
mod u64;
mod u8;
mod usize;
mod vec;

pub use super::*;
use egui::Ui;
pub use option::OptionDefault;

/// Options for using the default rendering style for the element. The options here are a superset
/// of all other options since "default" could be any of the widgets
///
/// So, not all elements will necessarily be used/respected. Use the non-default traits for typesafe
/// changes.
///
/// Marking a struct element with something like `#[debug_inspect(min_value = 5.0, max_value = 53.0)]`
/// will make the widget for that member default to those values.
#[derive(Clone, Default, Debug)]
pub struct InspectArgs {
    /// If true, the struct will have a visual/expandable header added to it. This defaults to true.
    ///
    /// To customize this, disable this header programmatically by passing your own
    /// InspectArgs into `render` or `render_mut`
    pub header: Option<bool>,

    /// If true, any child elements (i.e. struct members) will be indented. This defaults to true.
    pub indent_children: Option<bool>,

    /// Minimum value for the widget. The precise meaning of this can vary depending on the widget type
    pub min_value: Option<f32>,

    /// Maximum value for the widget. The precise meaning of this can vary depending on the widget type
    pub max_value: Option<f32>,

    /// Minimum value for the widget. The precise meaning of this can vary depending on the widget type
    pub step: Option<f32>,
}

/// Renders a value using the default widget
pub trait Inspect<T: ?Sized> {
    /// Render the element in an immutable way (i.e. static text)
    fn render(data: &T, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs);

    /// Render the element in a mutable way. Using this trait, the default widget to use is based
    /// on the type.
    fn render_mut(data: &mut T, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs)
        -> bool;
}

impl<T, I: Inspect<T>> Inspect<Box<T>> for Box<I> {
    fn render(data: &Box<T>, label: &'static str, ui: &mut Ui, args: &InspectArgs) {
        I::render(data, label, ui, args)
    }

    fn render_mut(data: &mut Box<T>, label: &'static str, ui: &mut Ui, args: &InspectArgs) -> bool {
        I::render_mut(data, label, ui, args)
    }
}
