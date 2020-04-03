mod default_bool;
mod default_f32;
mod default_option;
mod default_string;
mod default_u32;
mod default_usize;

pub use super::*;

/// Options for using the default rendering style for the element. The options here are a superset
/// of all other options since "default" could be any of the widgets
///
/// So, not all elements will necessarily be used/respected. Use the non-default traits for typesafe
/// changes.
///
/// Marking a struct element with something like `#[inspect(min_value = 5.0, max_value = 53.0)]`
/// will make the widget for that member default to those values.
#[derive(Debug, Default, Clone)]
pub struct InspectArgsDefault {
    /// If true, the struct will have a visual/expandable header added to it. This defaults to true.
    ///
    /// To customize this, disable this header programmatically by passing your own
    /// InspectArgsDefault into `render` or `render_mut`
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
pub trait InspectRenderDefault<T> {
    /// Render the element in an immutable way (i.e. static text)
    ///
    /// (Hopefully in the future this can be better. See
    /// https://github.com/ocornut/imgui/issues/211)
    fn render(
        data: &[&T],
        label: &'static str,
        world: &mut World,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    );

    /// Render the element in a mutable way. Using this trait, the default widget to use is based
    /// on the type.
    fn render_mut(
        data: &mut [&mut T],
        label: &'static str,
        world: &mut World,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool;
}
