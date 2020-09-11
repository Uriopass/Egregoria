use crate::{InspectArgsDefault, InspectRenderDefault};
use imgui::im_str;
use imgui::Ui;

mod geometry;

pub use geometry::*;

pub struct InspectDragf;

impl InspectRenderDefault<f32> for InspectDragf {
    fn render(data: &[&f32], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let cp = *data[0];
        ui.text(&im_str!("{} {}", cp, label));
    }

    fn render_mut(
        data: &mut [&mut f32],
        label: &'static str,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        ui.drag_float(&im_str!("{}", label), data[0])
            .speed(args.step.unwrap_or(0.1))
            .build()
    }
}

impl InspectRenderDefault<f64> for InspectDragf {
    fn render(data: &[&f64], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let cp = *data[0];
        ui.text(&im_str!("{} {}", cp, label));
    }

    fn render_mut(
        data: &mut [&mut f64],
        label: &'static str,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let mut cp = *data[0] as f32;
        let changed = ui
            .drag_float(&im_str!("{}", label), &mut cp)
            .speed(args.step.unwrap_or(0.1))
            .build();
        *data[0] = cp as f64;
        changed
    }
}
