use crate::gui::{ImCgVec2, ImEntity, ImVec};
use crate::rendering::colors::*;
use cgmath::num_traits::zero;
use cgmath::Vector2;
use imgui::Ui;
use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectRenderDefault;
use imgui_inspect_derive::*;
use specs::{Component, Entity, VecStorage};

pub enum MeshRenderEnum {
    Circle(CircleRender),
    Rect(RectRender),
    LineTo(LineToRender),
    Line(LineRender),
}

impl InspectRenderDefault<MeshRenderEnum> for MeshRenderEnum {
    fn render(
        _data: &[&MeshRenderEnum],
        _label: &'static str,
        _ui: &Ui,
        _args: &InspectArgsDefault,
    ) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut MeshRenderEnum],
        label: &'static str,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        if data.len() != 1 {
            return false;
        }
        let mre = &mut data[0];

        match mre {
            MeshRenderEnum::Circle(x) => {
                <CircleRender as InspectRenderDefault<CircleRender>>::render_mut(
                    &mut [x],
                    label,
                    ui,
                    args,
                )
            }
            MeshRenderEnum::Rect(x) => {
                <RectRender as InspectRenderDefault<RectRender>>::render_mut(
                    &mut [x],
                    label,
                    ui,
                    args,
                )
            }
            MeshRenderEnum::LineTo(x) => {
                <LineToRender as InspectRenderDefault<LineToRender>>::render_mut(
                    &mut [x],
                    label,
                    ui,
                    args,
                )
            }
            MeshRenderEnum::Line(x) => {
                <LineRender as InspectRenderDefault<LineRender>>::render_mut(
                    &mut [x],
                    label,
                    ui,
                    args,
                )
            }
        }
    }
}

impl From<CircleRender> for MeshRenderEnum {
    fn from(x: CircleRender) -> Self {
        MeshRenderEnum::Circle(x)
    }
}

impl From<RectRender> for MeshRenderEnum {
    fn from(x: RectRender) -> Self {
        MeshRenderEnum::Rect(x)
    }
}

impl From<LineToRender> for MeshRenderEnum {
    fn from(x: LineToRender) -> Self {
        MeshRenderEnum::LineTo(x)
    }
}

impl From<LineRender> for MeshRenderEnum {
    fn from(x: LineRender) -> Self {
        MeshRenderEnum::Line(x)
    }
}

#[derive(Component)]
#[storage(VecStorage)]
pub struct MeshRender {
    pub orders: Vec<MeshRenderEnum>,
}

impl InspectRenderDefault<MeshRender> for MeshRender {
    fn render(data: &[&MeshRender], label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        ui.indent();
        let mapped: Vec<&Vec<MeshRenderEnum>> = data.iter().map(|x| &x.orders).collect();
        <ImVec<MeshRenderEnum> as InspectRenderDefault<Vec<MeshRenderEnum>>>::render(
            &mapped, label, ui, args,
        );
        ui.unindent();
    }

    fn render_mut(
        data: &mut [&mut MeshRender],
        label: &'static str,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        ui.indent();
        let mut mapped: Vec<&mut Vec<MeshRenderEnum>> =
            data.into_iter().map(|x| &mut x.orders).collect();
        let changed =
            <ImVec<MeshRenderEnum> as InspectRenderDefault<Vec<MeshRenderEnum>>>::render_mut(
                &mut mapped,
                label,
                ui,
                args,
            );
        ui.unindent();
        changed
    }
}

impl<T: Into<MeshRenderEnum>> From<T> for MeshRender {
    fn from(x: T) -> Self {
        MeshRender::simple(x)
    }
}

impl<T: Into<MeshRenderEnum>, U: Into<MeshRenderEnum>> From<(T, U)> for MeshRender {
    fn from((x, y): (T, U)) -> Self {
        let mut m = MeshRender::simple(x);
        m.add(y);
        m
    }
}

impl<T: Into<MeshRenderEnum>, U: Into<MeshRenderEnum>, V: Into<MeshRenderEnum>> From<(T, U, V)>
    for MeshRender
{
    fn from((x, y, z): (T, U, V)) -> Self {
        let mut m = MeshRender::simple(x);
        m.add(y).add(z);
        m
    }
}

#[allow(dead_code)]
impl MeshRender {
    pub fn empty() -> Self {
        MeshRender { orders: vec![] }
    }

    pub fn add<T: Into<MeshRenderEnum>>(&mut self, x: T) -> &mut Self {
        self.orders.push(x.into());
        self
    }

    pub fn simple<T: Into<MeshRenderEnum>>(x: T) -> Self {
        MeshRender {
            orders: vec![x.into()],
        }
    }
}
#[derive(Inspect)]
pub struct CircleRender {
    #[inspect(proxy_type = "ImCgVec2")]
    pub offset: Vector2<f32>,
    pub radius: f32,
    pub color: Color,
    pub filled: bool,
}

impl Default for CircleRender {
    fn default() -> Self {
        CircleRender {
            offset: zero(),
            radius: 0.0,
            color: WHITE,
            filled: true,
        }
    }
}

#[derive(Inspect)]
pub struct RectRender {
    pub width: f32,
    pub height: f32,
    pub color: Color,
    pub filled: bool,
}

impl Default for RectRender {
    fn default() -> Self {
        RectRender {
            width: 0.0,
            height: 0.0,
            color: WHITE,
            filled: true,
        }
    }
}

#[derive(Inspect)]
pub struct LineToRender {
    #[inspect(proxy_type = "ImEntity")]
    pub to: Entity,
    pub color: Color,
}

#[derive(Inspect)]
pub struct LineRender {
    #[inspect(proxy_type = "ImCgVec2")]
    pub offset: Vector2<f32>,
    pub color: Color,
}
