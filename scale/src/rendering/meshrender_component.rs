use crate::geometry::Vec2;
use crate::gui::{ImEntity, InspectDragf, InspectVec, InspectVec2};
use crate::rendering::colors::*;
use cgmath::num_traits::zero;
use imgui::Ui;
use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectRenderDefault;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Component, DenseVecStorage, Entity, World};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshRenderEnum {
    StrokeCircle(StrokeCircleRender),
    Circle(CircleRender),
    Rect(RectRender),
    #[serde(skip)]
    LineTo(LineToRender),
    Line(LineRender),
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
    fn render(
        _: &[&MeshRenderEnum],
        _: &'static str,
        _: &mut World,
        _: &Ui,
        _: &InspectArgsDefault,
    ) {
        unimplemented!()
    }

    fn render_mut(
        data: &mut [&mut MeshRenderEnum],
        label: &'static str,
        world: &mut World,
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
            >>::render_mut(
                &mut [x], label, world, ui, args
            ),
            MeshRenderEnum::Circle(x) => {
                <CircleRender as InspectRenderDefault<CircleRender>>::render_mut(
                    &mut [x],
                    label,
                    world,
                    ui,
                    args,
                )
            }
            MeshRenderEnum::Rect(x) => {
                <RectRender as InspectRenderDefault<RectRender>>::render_mut(
                    &mut [x],
                    label,
                    world,
                    ui,
                    args,
                )
            }
            MeshRenderEnum::LineTo(x) => {
                <LineToRender as InspectRenderDefault<LineToRender>>::render_mut(
                    &mut [x],
                    label,
                    world,
                    ui,
                    args,
                )
            }
            MeshRenderEnum::Line(x) => {
                <LineRender as InspectRenderDefault<LineRender>>::render_mut(
                    &mut [x],
                    label,
                    world,
                    ui,
                    args,
                )
            }
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

#[derive(Clone, Serialize, Deserialize, Component)]
pub struct MeshRender {
    pub orders: Vec<MeshRenderEnum>,
    pub hide: bool,
    pub z: f32,
}

#[allow(dead_code)]
impl MeshRender {
    pub fn empty(z: f32) -> Self {
        MeshRender {
            orders: vec![],
            hide: false,
            z,
        }
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
    fn render(
        data: &[&MeshRender],
        label: &'static str,
        world: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) {
        let mapped: Vec<&Vec<MeshRenderEnum>> = data.iter().map(|x| &x.orders).collect();
        <InspectVec<MeshRenderEnum> as InspectRenderDefault<Vec<MeshRenderEnum>>>::render(
            &mapped, label, world, ui, args,
        );
    }

    fn render_mut(
        data: &mut [&mut MeshRender],
        label: &'static str,
        world: &mut World,
        ui: &Ui,
        args: &InspectArgsDefault,
    ) -> bool {
        let mut mapped: Vec<&mut Vec<MeshRenderEnum>> =
            data.iter_mut().map(|x| &mut x.orders).collect();
        <InspectVec<MeshRenderEnum> as InspectRenderDefault<Vec<MeshRenderEnum>>>::render_mut(
            &mut mapped,
            label,
            world,
            ui,
            args,
        )
    }
}

#[derive(Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct CircleRender {
    #[inspect(proxy_type = "InspectVec2")]
    pub offset: Vec2,
    #[inspect(proxy_type = "InspectDragf")]
    pub radius: f32,
    pub color: Color,
}

impl Default for CircleRender {
    fn default() -> Self {
        Self {
            offset: zero(),
            radius: 0.0,
            color: Color::WHITE,
        }
    }
}

#[derive(Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct StrokeCircleRender {
    #[inspect(proxy_type = "InspectVec2")]
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
            offset: zero(),
            radius: 0.0,
            color: Color::WHITE,
            thickness: 0.1,
        }
    }
}

#[derive(Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct RectRender {
    #[inspect(proxy_type = "InspectVec2")]
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

#[derive(Debug, Inspect, Clone)]
pub struct LineToRender {
    #[inspect(proxy_type = "ImEntity")]
    pub to: Entity,
    pub color: Color,
    #[inspect(proxy_type = "InspectDragf")]
    pub thickness: f32,
}

#[derive(Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct LineRender {
    #[inspect(proxy_type = "InspectVec2")]
    pub offset: Vec2,
    pub color: Color,
    #[inspect(proxy_type = "InspectDragf")]
    pub thickness: f32,
}
