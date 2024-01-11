use crate::impls::f64::InspectF64Deg;
use crate::impls::Inspect;
use crate::impls::InspectArgs;
use egui::color_picker::Alpha;
use egui::Color32;
use geom::{Color, LinearColor, PolyLine, Transform, Vec2, Vec3};

impl Inspect<Color> for Color {
    fn render(c: &Self, label: &'static str, ui: &mut egui::Ui, _args: &InspectArgs) {
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
        _args: &InspectArgs,
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

impl Inspect<LinearColor> for LinearColor {
    fn render(lc: &Self, label: &'static str, ui: &mut egui::Ui, _: &InspectArgs) {
        let c: Color = (*lc).into();
        <Color as Inspect<Color>>::render(&c, label, ui, &InspectArgs::default());
    }

    fn render_mut(
        lc: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgs,
    ) -> bool {
        let mut c: Color = (*lc).into();
        if <Color as Inspect<Color>>::render_mut(&mut c, label, ui, &InspectArgs::default()) {
            *lc = c.into();
            true
        } else {
            false
        }
    }
}

impl Inspect<Transform> for Transform {
    fn render(t: &Self, _: &'static str, ui: &mut egui::Ui, _: &InspectArgs) {
        let position = t.pos;
        let direction = t.dir;
        <Vec3 as Inspect<Vec3>>::render(&position, "position", ui, &InspectArgs::default());
        <Vec3 as Inspect<Vec3>>::render(&direction, "direction", ui, &InspectArgs::default());
    }

    fn render_mut(t: &mut Self, _: &'static str, ui: &mut egui::Ui, _: &InspectArgs) -> bool {
        let mut position = t.pos;
        let mut direction = t.dir;
        let mut changed = <Vec3 as Inspect<Vec3>>::render_mut(
            &mut position,
            "position",
            ui,
            &InspectArgs::default(),
        );
        changed |= <Vec3 as Inspect<Vec3>>::render_mut(
            &mut direction,
            "direction",
            ui,
            &InspectArgs::default(),
        );
        t.dir = direction.normalize();
        t.pos = position;
        changed
    }
}

pub struct InspectVec2Immutable;
impl Inspect<Vec2> for InspectVec2Immutable {
    fn render(v: &Vec2, label: &'static str, ui: &mut egui::Ui, _: &InspectArgs) {
        ui.horizontal(|ui| {
            ui.label(label);
            <f32 as Inspect<f32>>::render(&v.x, "", ui, &InspectArgs::default());
            <f32 as Inspect<f32>>::render(&v.y, "", ui, &InspectArgs::default());
        });
    }

    fn render_mut(
        v: &mut Vec2,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        Self::render(&*v, label, ui, args);
        false
    }
}

impl Inspect<Vec2> for Vec2 {
    fn render(v: &Vec2, label: &'static str, ui: &mut egui::Ui, _: &InspectArgs) {
        <InspectVec2Immutable as Inspect<Vec2>>::render(v, label, ui, &InspectArgs::default());
    }

    fn render_mut(
        v: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgs,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(label);
            changed |= <f32 as Inspect<f32>>::render_mut(&mut v.x, "", ui, &InspectArgs::default());
            changed |= <f32 as Inspect<f32>>::render_mut(&mut v.y, "", ui, &InspectArgs::default());
        });
        changed
    }
}

impl Inspect<Vec3> for Vec3 {
    fn render(v: &Self, label: &'static str, ui: &mut egui::Ui, _: &InspectArgs) {
        ui.horizontal(|ui| {
            ui.label(label);
            <f32 as Inspect<f32>>::render(&v.x, "", ui, &InspectArgs::default());
            <f32 as Inspect<f32>>::render(&v.y, "", ui, &InspectArgs::default());
            <f32 as Inspect<f32>>::render(&v.z, "", ui, &InspectArgs::default());
        });
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        _args: &InspectArgs,
    ) -> bool {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(label);
            changed |=
                <f32 as Inspect<f32>>::render_mut(&mut data.x, "", ui, &InspectArgs::default());
            changed |=
                <f32 as Inspect<f32>>::render_mut(&mut data.y, "", ui, &InspectArgs::default());
            changed |=
                <f32 as Inspect<f32>>::render_mut(&mut data.z, "", ui, &InspectArgs::default());
        });
        changed
    }
}

impl Inspect<PolyLine> for PolyLine {
    fn render(data: &Self, label: &'static str, ui: &mut egui::Ui, args: &InspectArgs) {
        <[Vec2] as Inspect<[Vec2]>>::render(data.as_slice(), label, ui, args);
    }

    fn render_mut(
        data: &mut Self,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        <[Vec2] as Inspect<[Vec2]>>::render_mut(data.as_mut_slice(), label, ui, args)
    }
}

pub struct InspectVec2Rotation;
impl Inspect<Vec2> for InspectVec2Rotation {
    fn render(v: &Vec2, label: &'static str, ui: &mut egui::Ui, _: &InspectArgs) {
        let ang = f32::atan2(v.y, v.x);
        <f32 as Inspect<f32>>::render(&ang, label, ui, &InspectArgs::default());
    }

    fn render_mut(
        data: &mut Vec2,
        label: &'static str,
        ui: &mut egui::Ui,
        args: &InspectArgs,
    ) -> bool {
        let mut changed = false;
        let mut ang: f64 = f64::atan2(data.y as f64, data.x as f64) * 180.0 / std::f64::consts::PI;
        changed |= <InspectF64Deg as Inspect<f64>>::render_mut(&mut ang, label, ui, args);
        let ang = ang * std::f64::consts::PI / 180.0;
        data.x = f64::cos(ang) as f32;
        data.y = f64::sin(ang) as f32;
        changed
    }
}
