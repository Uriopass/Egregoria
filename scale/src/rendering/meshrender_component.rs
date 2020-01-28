use crate::engine_interaction::MAX_LAYERS;
use crate::gui::{ImCgVec2, ImDragf, ImEntity, ImVec};
use crate::rendering::colors::*;
use cgmath::num_traits::zero;
use cgmath::Vector2;
use imgui::Ui;
use imgui_inspect::InspectArgsDefault;
use imgui_inspect::InspectRenderDefault;
use imgui_inspect_derive::*;
use serde::{Deserialize, Serialize};
use specs::{Component, Entity, FlaggedStorage, VecStorage, World};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshRenderEnum {
    Circle(CircleRender),
    Rect(RectRender),
    #[serde(skip)]
    LineTo(LineToRender),
    Line(LineRender),
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

#[derive(Clone, Serialize, Deserialize)]
pub struct MeshRender {
    pub orders: Vec<MeshRenderEnum>,
    pub hide: bool,
    layer: u32,
}

#[allow(dead_code)]
impl MeshRender {
    pub fn empty(layer: u32) -> Self {
        if layer >= MAX_LAYERS {
            panic!("Invalid layer: {}", layer);
        }
        MeshRender {
            orders: vec![],
            hide: false,
            layer,
        }
    }

    pub fn layer(&self) -> u32 {
        self.layer
    }

    pub fn add<T: Into<MeshRenderEnum>>(&mut self, x: T) -> &mut Self {
        self.orders.push(x.into());
        self
    }

    pub fn simple<T: Into<MeshRenderEnum>>(x: T, layer: u32) -> Self {
        if layer >= MAX_LAYERS {
            panic!("Invalid layer: {}", layer);
        }
        MeshRender {
            orders: vec![x.into()],
            hide: false,
            layer,
        }
    }

    pub fn build(self) -> Self {
        self
    }
}

impl Component for MeshRender {
    type Storage = FlaggedStorage<Self, VecStorage<Self>>;
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
        <ImVec<MeshRenderEnum> as InspectRenderDefault<Vec<MeshRenderEnum>>>::render(
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
        <ImVec<MeshRenderEnum> as InspectRenderDefault<Vec<MeshRenderEnum>>>::render_mut(
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
    #[inspect(proxy_type = "ImCgVec2")]
    pub offset: Vector2<f32>,
    #[inspect(proxy_type = "ImDragf")]
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

#[derive(Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct RectRender {
    #[inspect(proxy_type = "ImDragf")]
    pub width: f32,
    #[inspect(proxy_type = "ImDragf")]
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

#[derive(Debug, Inspect, Clone)]
pub struct LineToRender {
    #[inspect(proxy_type = "ImEntity")]
    pub to: Entity,
    pub color: Color,
    #[inspect(proxy_type = "ImDragf")]
    pub thickness: f32,
}

#[derive(Debug, Inspect, Clone, Serialize, Deserialize)]
pub struct LineRender {
    #[inspect(proxy_type = "ImCgVec2")]
    pub offset: Vector2<f32>,
    pub color: Color,
    #[inspect(proxy_type = "ImDragf")]
    pub thickness: f32,
}
