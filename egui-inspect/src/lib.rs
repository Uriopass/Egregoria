mod impls;

pub use egui;
pub use impls::geometry::*;
pub use impls::*;

#[cfg(feature = "derive")]
pub use egui_inspect_derive::Inspect;

#[rustfmt::skip]
#[macro_export]
macro_rules! debug_inspect_impl {
    ($t: ty) => {
        impl egui_inspect::Inspect<$t> for $t {
            fn render(
                data: &$t,
                label: &'static str,
                ui: &mut egui_inspect::egui::Ui,
                _: &egui_inspect::InspectArgs,
            ) {
                let d = data;
                if label == "" {
                    ui.label(format!("{d:?}"));
                } else {
                    ui.label(format!("{label}: {d:?}"));
                }
            }

            fn render_mut(
                data: &mut $t,
                label: &'static str,
                ui: &mut egui_inspect::egui::Ui,
                _: &egui_inspect::InspectArgs,
            ) -> bool {
                let d = data;
                if label == "" {
                    ui.label(format!("{d:?}"));
                } else {
                    ui.label(format!("{label}: {d:?}"));
                }
                false
            }
        }
    };
}
