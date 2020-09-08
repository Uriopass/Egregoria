use crate::default::InspectArgsDefault;
use crate::default::InspectRenderDefault;
use geom::{PolyLine, Vec2};
use imgui::im_str;

impl InspectRenderDefault<Vec2> for Vec2 {
    fn render(data: &[&Vec2], label: &'static str, ui: &imgui::Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        imgui::InputFloat2::new(ui, &im_str!("{}", label), &mut [x.x, x.y])
            .always_insert_mode(false)
            .build();
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut conv = [x.x, x.y];
        let changed = ui
            .drag_float2(&im_str!("{}", label), &mut conv)
            .speed(args.step.unwrap_or(0.1))
            .build();
        x.x = conv[0];
        x.y = conv[1];
        changed
    }
}

impl InspectRenderDefault<PolyLine> for PolyLine {
    fn render(
        _data: &[&PolyLine],
        _label: &'static str,
        _ui: &imgui::Ui,
        _args: &InspectArgsDefault,
    ) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut PolyLine],
        label: &str,
        ui: &imgui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = &mut data[0];
        let mut changed = false;

        if imgui::CollapsingHeader::new(&im_str!("{}", label)).build(&ui) {
            ui.indent();
            for (i, x) in v.iter_mut().enumerate() {
                let id = ui.push_id(i as i32);
                changed |= <Vec2 as InspectRenderDefault<Vec2>>::render_mut(&mut [x], "", ui, args);
                id.pop(ui);
            }
            ui.unindent();
        }

        changed
    }
}
