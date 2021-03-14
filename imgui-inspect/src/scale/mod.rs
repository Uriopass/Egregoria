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
        let mut cp = *data[0];
        imgui::InputFloat::new(ui, &*imgui::im_str!("{}", label), &mut cp)
            .read_only(true)
            .build();
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
        imgui::Drag::new(&im_str!("{}", label))
            .speed(args.step.unwrap_or(0.1))
            .build(ui, data[0])
    }
}

impl InspectRenderDefault<f64> for InspectDragf {
    fn render(data: &[&f64], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let mut cp = *data[0] as f32;
        imgui::InputFloat::new(ui, &*imgui::im_str!("{}", label), &mut cp)
            .read_only(true)
            .build();
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
        let changed = imgui::Drag::new(&im_str!("{}", label))
            .speed(args.step.unwrap_or(0.1))
            .build(ui, &mut cp);
        *data[0] = cp as f64;
        changed
    }
}
