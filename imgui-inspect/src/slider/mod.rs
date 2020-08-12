mod slider_f32;

pub use super::*;

/// Options for rendering a values as a slider.
///
/// Marking a struct member will give it a default rendering behavior. For example,
/// `#[inspect_slider(min_value = 5.0, max_value = 53.0)]`
#[derive(Debug, Default)]
pub struct InspectArgsSlider {
    /// The minimum value for the slider
    pub min_value: Option<f32>,

    /// The maximum value on the slider
    pub max_value: Option<f32>,
}

impl From<InspectArgsDefault> for InspectArgsSlider {
    fn from(default_args: InspectArgsDefault) -> Self {
        Self {
            min_value: default_args.min_value,
            max_value: default_args.max_value,
        }
    }
}

/// Renders a value as a slider
pub trait InspectRenderSlider<T> {
    /// Render the element in an immutable way (i.e. static text)
    ///
    /// (Hopefully in the future this can be better. See
    /// https://github.com/ocornut/imgui/issues/211)
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui, args: &InspectArgsSlider);

    /// Render the element as a mutable slider
    fn render_mut(
        data: &mut [&mut T],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsSlider,
    ) -> bool;
}
