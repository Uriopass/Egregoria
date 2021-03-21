use geom::{Color, Vec2};
use imgui::Ui;
use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectDragf;
use imgui_inspect::InspectRenderDefault;
use imgui_inspect_derive::*;
use legion::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MeshRenderEnum {
    StrokeCircle(StrokeCircleRender),
    Circle(CircleRender),
    Rect(RectRender),
    #[serde(skip)]
    LineTo(LineToRender),
    Line(LineRender),
    AbsoluteLine(AbsoluteLineRender),
}

impl MeshRenderEnum {
    pub fn as_circle_mut(&mut self) -> &mut CircleRender {
        match self {
            MeshRenderEnum::Circle(x) => x,
            _ => panic!(),
        }
    }

    pub fn as_rect_mut(&mut self) -> &mut RectRender {
        match self {
            MeshRenderEnum::Rect(x) => x,
            _ => panic!(),
        }
    }
}

impl InspectRenderDefault<MeshRenderEnum> for MeshRenderEnum {
    fn render(data: &[&MeshRenderEnum], label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        let mre = data[0];

        match mre {
            MeshRenderEnum::StrokeCircle(x) => <StrokeCircleRender as InspectRenderDefault<
                StrokeCircleRender,
            >>::render(&[x], label, ui, args),
            MeshRenderEnum::Circle(x) => {
                <CircleRender as InspectRenderDefault<CircleRender>>::render(&[x], label, ui, args)
            }
            MeshRenderEnum::Rect(x) => {
                <RectRender as InspectRenderDefault<RectRender>>::render(&[x], label, ui, args)
            }
            MeshRenderEnum::LineTo(x) => {
                <LineToRender as InspectRenderDefault<LineToRender>>::render(&[x], label, ui, args)
            }
            MeshRenderEnum::Line(x) => {
                <LineRender as InspectRenderDefault<LineRender>>::render(&[x], label, ui, args)
            }
            MeshRenderEnum::AbsoluteLine(x) => <AbsoluteLineRender as InspectRenderDefault<
                AbsoluteLineRender,
            >>::render(&[x], label, ui, args),
        }
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
            MeshRenderEnum::StrokeCircle(x) => <StrokeCircleRender as InspectRenderDefault<
                StrokeCircleRender,
            >>::render_mut(&mut [x], label, ui, args),
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
            MeshRenderEnum::AbsoluteLine(x) => <AbsoluteLineRender as InspectRenderDefault<
                AbsoluteLineRender,
            >>::render_mut(&mut [x], label, ui, args),
        }
    }
}

macro_rules! mk_from_mr {
    ($t: ty; $p: expr) => {
        impl From<$t> for MeshRenderEnum {
            fn from(x: $t) -> Self {
                $p(x)
            }
        }
    };
}

mk_from_mr!(StrokeCircleRender; |x| MeshRenderEnum::StrokeCircle(x));
mk_from_mr!(CircleRender; |x| MeshRenderEnum::Circle(x));
mk_from_mr!(RectRender; |x| MeshRenderEnum::Rect(x));
mk_from_mr!(LineRender; |x| MeshRenderEnum::Line(x));
mk_from_mr!(LineToRender; |x| MeshRenderEnum::LineTo(x));
mk_from_mr!(AbsoluteLineRender; |x| MeshRenderEnum::AbsoluteLine(x));

#[derive(Clone, Serialize, Deserialize)]
pub struct MeshRender {
    pub orders: Vec<MeshRenderEnum>,
    pub hide: bool,
    pub z: f32,
}

impl MeshRender {
    pub fn empty(z: f32) -> Self {
        MeshRender {
            orders: vec![],
            hide: false,
            z,
        }
    }

    pub fn hidden(&mut self) -> &mut Self {
        self.hide = true;
        self
    }

    pub fn add<T: Into<MeshRenderEnum>>(&mut self, x: T) -> &mut Self {
        self.orders.push(x.into());
        self
    }

    pub fn simple<T: Into<MeshRenderEnum>>(x: T, z: f32) -> Self {
        MeshRender {
            orders: vec![x.into()],
            hide: false,
            z,
        }
    }

    pub fn build(&self) -> Self {
        self.clone()
    }
}

impl InspectRenderDefault<MeshRender> for MeshRender {
    fn render(data: &[&MeshRender], label: &'static str, ui: &Ui, args: &InspectArgsDefault) {
        if data.len() != 1 {
            panic!()
        }
        let mapped = &data[0].orders;
        <Vec<MeshRenderEnum> as InspectRenderDefault<Vec<MeshRenderEnum>>>::render(
            &[mapped],
            label,
            ui,
            args,
        );
    }

    fn render_mut(
        data: &mut [&mut MeshRender],
        label: &'static str,

        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let mut mapped: Vec<&mut Vec<MeshRenderEnum>> =
            data.iter_mut().map(|x| &mut x.orders).collect();
        <Vec<MeshRenderEnum> as InspectRenderDefault<Vec<MeshRenderEnum>>>::render_mut(
            &mut mapped,
            label,
            ui,
            args,
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Inspect)]
pub struct CircleRender {
    pub offset: Vec2,
    #[inspect(proxy_type = "InspectDragf")]
    pub radius: f32,
    pub color: Color,
}

impl Default for CircleRender {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            radius: 0.0,
            color: Color::WHITE,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Inspect)]
pub struct StrokeCircleRender {
    pub offset: Vec2,
    #[inspect(proxy_type = "InspectDragf")]
    pub radius: f32,
    pub color: Color,
    #[inspect(proxy_type = "InspectDragf")]
    pub thickness: f32,
}

impl Default for StrokeCircleRender {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            radius: 0.0,
            color: Color::WHITE,
            thickness: 0.1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Inspect)]
pub struct RectRender {
    pub offset: Vec2,
    #[inspect(proxy_type = "InspectDragf")]
    pub width: f32,
    #[inspect(proxy_type = "InspectDragf")]
    pub height: f32,
    pub color: Color,
}

impl Default for RectRender {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0].into(),
            width: 0.0,
            height: 0.0,
            color: Color::WHITE,
        }
    }
}

#[derive(Clone, Debug, Inspect)]
pub struct LineToRender {
    #[inspect(skip)]
    pub to: Entity,
    pub color: Color,
    #[inspect(proxy_type = "InspectDragf")]
    pub thickness: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Inspect)]
pub struct LineRender {
    pub offset: Vec2,
    pub color: Color,
    #[inspect(proxy_type = "InspectDragf")]
    pub thickness: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Inspect)]
pub struct AbsoluteLineRender {
    pub src: Vec2,
    pub dst: Vec2,
    pub color: Color,
    #[inspect(proxy_type = "InspectDragf")]
    pub thickness: f32,
}
