use crate::default::InspectArgsDefault;
use crate::default::InspectRenderDefault;
use geom::{from_srgb, Color, LinearColor, PolyLine, Transform, Vec2};
use imgui::{im_str, ColorEdit, EditableColor, Ui};

impl InspectRenderDefault<Color> for Color {
    fn render(data: &[&Color], label: &'static str, ui: &Ui, _args: &InspectArgsDefault) {
        let c = data[0];
        let mut color_arr = [c.r, c.g, c.b, c.a];
        ColorEdit::new(&im_str!("{}", label), EditableColor::Float4(&mut color_arr)).build(ui);
    }

    fn render_mut(
        data: &mut [&mut Color],
        label: &'static str,
        ui: &Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let c = &mut data[0];
        let mut color_arr = [c.r, c.g, c.b, c.a];
        if ColorEdit::new(&im_str!("{}", label), EditableColor::Float4(&mut color_arr)).build(ui) {
            c.r = color_arr[0];
            c.g = color_arr[1];
            c.b = color_arr[2];
            c.a = color_arr[3];
            true
        } else {
            false
        }
    }
}

impl InspectRenderDefault<LinearColor> for LinearColor {
    fn render(data: &[&LinearColor], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        let lc = data[0];
        let c: Color = (*lc).into();
        let mut color_arr = [c.r, c.g, c.b, c.a];
        ColorEdit::new(&im_str!("{}", label), EditableColor::Float4(&mut color_arr)).build(ui);
    }

    fn render_mut(
        data: &mut [&mut LinearColor],
        label: &'static str,
        ui: &Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }

        let lc = &mut *data[0];
        let c: Color = (*lc).into();
        let mut color_arr = [c.r, c.g, c.b, c.a];
        if ColorEdit::new(&im_str!("{}", label), EditableColor::Float4(&mut color_arr)).build(ui) {
            lc.r = from_srgb(color_arr[0]);
            lc.g = from_srgb(color_arr[1]);
            lc.b = from_srgb(color_arr[2]);
            lc.a = color_arr[3];
            true
        } else {
            false
        }
    }
}

impl InspectRenderDefault<Transform> for Transform {
    fn render(data: &[&Transform], _: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        let t = data[0];
        let position = t.position();
        let direction = t.direction();
        <Vec2 as InspectRenderDefault<Vec2>>::render(
            &[&position],
            "position",
            ui,
            &InspectArgsDefault::default(),
        );
        <InspectVec2Rotation as InspectRenderDefault<Vec2>>::render(
            &[&direction],
            "direction",
            ui,
            &InspectArgsDefault::default(),
        );
    }

    fn render_mut(
        data: &mut [&mut Transform],
        _: &'static str,
        ui: &Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut position = x.position();
        let mut direction = x.direction();
        let mut changed = <Vec2 as InspectRenderDefault<Vec2>>::render_mut(
            &mut [&mut position],
            "position",
            ui,
            &InspectArgsDefault::default(),
        );
        changed |= <InspectVec2Rotation as InspectRenderDefault<Vec2>>::render_mut(
            &mut [&mut direction],
            "direction",
            ui,
            &InspectArgsDefault::default(),
        );
        x.set_direction(direction);
        x.set_position(position);
        changed
    }
}

pub struct InspectVec2Immutable;
impl InspectRenderDefault<Vec2> for InspectVec2Immutable {
    fn render(data: &[&Vec2], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
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

        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        Self::render(&[&*data[0]], label, ui, args);
        false
    }
}

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
        let changed = imgui::Drag::new(&im_str!("{}", label))
            .speed(args.step.unwrap_or(0.1))
            .build_array(ui, &mut conv);
        x.x = conv[0];
        x.y = conv[1];
        changed
    }
}

impl InspectRenderDefault<PolyLine> for PolyLine {
    fn render(data: &[&PolyLine], label: &'static str, ui: &imgui::Ui, args: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }

        let v = data[0];
        if imgui::CollapsingHeader::new(&im_str!("{}", label)).build(&ui) {
            ui.indent();
            for (i, x) in v.iter().enumerate() {
                let id = ui.push_id(i as i32);
                <Vec2 as InspectRenderDefault<Vec2>>::render(&[x], "", ui, args);
                id.pop(ui);
            }
            ui.unindent();
        }
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

        let v = &mut *data[0];
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

pub struct InspectVec2Rotation;
impl InspectRenderDefault<Vec2> for InspectVec2Rotation {
    fn render(data: &[&Vec2], label: &'static str, ui: &Ui, _: &InspectArgsDefault) {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = data[0];
        let mut ang = f32::atan2(x.y, x.x);
        imgui::InputFloat::new(ui, &*im_str!("{}", label), &mut ang)
            .read_only(true)
            .build();
    }

    fn render_mut(
        data: &mut [&mut Vec2],
        label: &'static str,

        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            unimplemented!();
        }
        let x = &mut data[0];
        let mut ang = f32::atan2(x.y, x.x);

        let changed = imgui::Drag::new(&im_str!("{}", label))
            .speed(-args.step.unwrap_or(0.1))
            .build(ui, &mut ang);
        x.x = ang.cos();
        x.y = ang.sin();
        changed
    }
}
