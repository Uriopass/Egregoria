use crate::{InspectArgsDefault, InspectRenderDefault};
use imgui::Ui;

mod geometry;

pub use geometry::*;

pub struct InspectDragf;

impl InspectRenderDefault<f32> for InspectDragf {
    fn render(data: &[&f32], label: &'static str, ui: &Ui<'_>, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let mut cp = *data[0];
        imgui::InputFloat::new(ui, &*label, &mut cp)
            .read_only(true)
            .build();
    }

    fn render_mut(
        data: &mut [&mut f32],
        label: &'static str,
        ui: &Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let format = if args.step == Some(1.0) {
            "%.0f"
        } else if args.step == Some(0.1) {
            "%.1f"
        } else {
            "%f"
        };

        imgui::Drag::new(&label)
            .speed(args.step.unwrap_or(0.1))
            .display_format(&*format)
            .range(
                args.min_value.unwrap_or(f32::NEG_INFINITY),
                args.max_value.unwrap_or(f32::INFINITY),
            )
            .build(ui, data[0])
    }
}

impl InspectRenderDefault<f64> for InspectDragf {
    fn render(data: &[&f64], label: &'static str, ui: &Ui<'_>, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let mut cp = *data[0] as f32;
        imgui::InputFloat::new(ui, &*label, &mut cp)
            .read_only(true)
            .build();
    }

    fn render_mut(
        data: &mut [&mut f64],
        label: &'static str,
        ui: &Ui<'_>,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let mut cp = *data[0] as f32;
        let changed = imgui::Drag::new(&label)
            .speed(args.step.unwrap_or(0.1))
            .build(ui, &mut cp);
        *data[0] = cp as f64;
        changed
    }
}
