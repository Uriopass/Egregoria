use crate::rendering::colors::*;
use cgmath::num_traits::zero;
use cgmath::Vector2;
use specs::{Component, Entity, VecStorage};

pub enum MeshRenderEnum {
    Circle(CircleRender),
    Rect(RectRender),
    LineTo(LineToRender),
    Line(LineRender),
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
pub struct MeshRenderComponent {
    pub orders: Vec<MeshRenderEnum>,
}

impl<T: Into<MeshRenderEnum>> From<T> for MeshRenderComponent {
    fn from(x: T) -> Self {
        MeshRenderComponent::simple(x)
    }
}

impl<T: Into<MeshRenderEnum>, U: Into<MeshRenderEnum>> From<(T, U)> for MeshRenderComponent {
    fn from((x, y): (T, U)) -> Self {
        let mut m = MeshRenderComponent::simple(x);
        m.add(y);
        m
    }
}

impl<T: Into<MeshRenderEnum>, U: Into<MeshRenderEnum>, V: Into<MeshRenderEnum>> From<(T, U, V)>
    for MeshRenderComponent
{
    fn from((x, y, z): (T, U, V)) -> Self {
        let mut m = MeshRenderComponent::simple(x);
        m.add(y).add(z);
        m
    }
}

#[allow(dead_code)]
impl MeshRenderComponent {
    pub fn empty() -> Self {
        MeshRenderComponent { orders: vec![] }
    }

    pub fn add<T: Into<MeshRenderEnum>>(&mut self, x: T) -> &mut Self {
        self.orders.push(x.into());
        self
    }

    pub fn simple<T: Into<MeshRenderEnum>>(x: T) -> Self {
        MeshRenderComponent {
            orders: vec![x.into()],
        }
    }
}

pub struct CircleRender {
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

pub struct LineToRender {
    pub to: Entity,
    pub color: Color,
}

pub struct LineRender {
    pub offset: Vector2<f32>,
    pub color: Color,
}
