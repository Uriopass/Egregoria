use crate::default::InspectArgsDefault;
use crate::default::InspectRenderDefault;
use egui::color_picker::Alpha;
use egui::Color32;
use geom::{Color, LinearColor, PolyLine, Transform, Vec2, Vec3};

impl InspectRenderDefault<Color> for Color {
    fn render(c: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgsDefault) {
        let mut color_arr = Color32::from_rgba_unmultiplied(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            (c.a * 255.0) as u8,
        );
        ui.horizontal(|ui| {
            ui.label(label);
            egui::color_picker::color_picker_color32(ui, &mut color_arr, Alpha::OnlyBlend);
        });
    }

    fn render_mut(
        c: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let mut color_arr = Color32::from_rgba_unmultiplied(
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
            (c.a * 255.0) as u8,
        );
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(label);
            if egui::color_picker::color_picker_color32(ui, &mut color_arr, Alpha::OnlyBlend) {
                let arr = color_arr.to_srgba_unmultiplied();
                c.r = arr[0] as f32 / 255.0;
                c.g = arr[1] as f32 / 255.0;
                c.b = arr[2] as f32 / 255.0;
                c.a = arr[3] as f32 / 255.0;
                changed = true;
            }
        });
        changed
    }
}

impl InspectRenderDefault<LinearColor> for LinearColor {
    fn render(lc: &Self, label: &'static str, ui: &mut egui::Ui, _: &InspectArgsDefault) {
        let c: Color = (*lc).into();
        <Color as InspectRenderDefault<Color>>::render(
            &c,
            label,
            ui,
            &InspectArgsDefault::default(),
        );
    }

    fn render_mut(
        lc: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let mut c: Color = (*lc).into();
        if <Color as InspectRenderDefault<Color>>::render_mut(
            &mut c,
            label,
            ui,
            &InspectArgsDefault::default(),
        ) {
            *lc = c.into();
            true
        } else {
            false
        }
    }
}

impl InspectRenderDefault<Transform> for Transform {
    fn render(t: &Self, _: &'static str, ui: &mut egui::Ui, _: &InspectArgsDefault) {
        let position = t.position;
        let direction = t.dir;
        <Vec3 as InspectRenderDefault<Vec3>>::render(
            &position,
            "position",
            ui,
            &InspectArgsDefault::default(),
        );
        <Vec3 as InspectRenderDefault<Vec3>>::render(
            &direction,
            "direction",
            ui,
            &InspectArgsDefault::default(),
        );
    }

    fn render_mut(
        t: &mut Self,
        _: &'static str,
        ui: &mut egui::Ui,
        _: &InspectArgsDefault,
    ) -> bool {
        let mut position = t.position;
        let mut direction = t.dir;
        let mut changed = <Vec3 as InspectRenderDefault<Vec3>>::render_mut(
            &mut position,
            "position",
            ui,
            &InspectArgsDefault::default(),
        );
        changed |= <Vec3 as InspectRenderDefault<Vec3>>::render_mut(
            &mut direction,
            "direction",
            ui,
            &InspectArgsDefault::default(),
        );
        t.dir = direction.normalize();
        t.position = position;
        changed
    }
}

pub struct InspectVec2Immutable;
impl InspectRenderDefault<Vec2> for InspectVec2Immutable {
    fn render(v: &Vec2, label: &'static str, ui: &mut egui::Ui, _: &InspectArgsDefault) {
        ui.horizontal(|ui| {
            ui.label(label);
            <f32 as InspectRenderDefault<f32>>::render(
                &v.x,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
            <f32 as InspectRenderDefault<f32>>::render(
                &v.y,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
        });
    }

    fn render_mut(
        v: &mut Vec2,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        Self::render(&*v, label, ui, args);
        false
    }
}

impl InspectRenderDefault<Vec2> for Vec2 {
    fn render(v: &Vec2, label: &'static str, ui: &mut egui::Ui, _: &InspectArgsDefault) {
        <InspectVec2Immutable as InspectRenderDefault<Vec2>>::render(
            v,
            label,
            ui,
            &InspectArgsDefault::default(),
        );
    }

    fn render_mut(
        v: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(label);
            changed |= <f32 as InspectRenderDefault<f32>>::render_mut(
                &mut v.x,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
            changed |= <f32 as InspectRenderDefault<f32>>::render_mut(
                &mut v.y,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
        });
        changed
    }
}

impl InspectRenderDefault<Vec3> for Vec3 {
    fn render(v: &Self, label: &'static str, ui: &mut egui::Ui, _: &InspectArgsDefault) {
        ui.horizontal(|ui| {
            ui.label(label);
            <f32 as InspectRenderDefault<f32>>::render(
                &v.x,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
            <f32 as InspectRenderDefault<f32>>::render(
                &v.y,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
            <f32 as InspectRenderDefault<f32>>::render(
                &v.z,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
        });
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(label);
            changed |= <f32 as InspectRenderDefault<f32>>::render_mut(
                &mut data.x,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
            changed |= <f32 as InspectRenderDefault<f32>>::render_mut(
                &mut data.y,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
            changed |= <f32 as InspectRenderDefault<f32>>::render_mut(
                &mut data.z,
                "",
                ui,
                &InspectArgsDefault::default(),
            );
        });
        changed
    }
}

impl InspectRenderDefault<PolyLine> for PolyLine {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgsDefault) {
        <[Vec2] as InspectRenderDefault<[Vec2]>>::render(data.as_slice(), label, ui, args);
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        <[Vec2] as InspectRenderDefault<[Vec2]>>::render_mut(data.as_mut_slice(), label, ui, args)
    }
}

pub struct InspectVec2Rotation;
impl InspectRenderDefault<Vec2> for InspectVec2Rotation {
    fn render(v: &Vec2, label: &'static str, ui: &mut egui::Ui, _: &InspectArgsDefault) {
        let ang = f32::atan2(v.y, v.x);
        <f32 as InspectRenderDefault<f32>>::render(&ang, label, ui, &InspectArgsDefault::default());
    }

    fn render_mut(
        data: &mut Vec2,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgsDefault,
    ) -> bool {
        let mut changed = false;
        let mut ang = f32::atan2(data.y, data.x);
        changed |= <f32 as InspectRenderDefault<f32>>::render_mut(
            &mut ang,
            label,
            ui,
            &InspectArgsDefault::default(),
        );
        data.x = f32::cos(ang);
        data.y = f32::sin(ang);
        changed
    }
}
