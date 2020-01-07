use crate::components::Transform;
use crate::rendering::render_context::RenderContext;
use cgmath::num_traits::zero;
use cgmath::Vector2;
use ggez::graphics::{Color, WHITE};
use specs::{Component, Entity, ReadStorage, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct MeshRenderComponent {
    pub orders: Vec<Box<dyn MeshRenderable>>,
}

#[allow(dead_code)]
impl MeshRenderComponent {
    pub fn empty() -> Self {
        MeshRenderComponent { orders: vec![] }
    }

    pub fn add<T: 'static + MeshRenderable>(&mut self, x: T) -> &mut Self {
        self.orders.push(Box::new(x));
        self
    }

    pub fn simple<T: 'static + MeshRenderable>(x: T) -> Self {
        MeshRenderComponent {
            orders: vec![Box::new(x)],
        }
    }
}

impl<T: 'static + MeshRenderable> From<T> for MeshRenderComponent {
    fn from(x: T) -> Self {
        MeshRenderComponent::simple(x)
    }
}

impl<T: 'static + MeshRenderable, U: 'static + MeshRenderable> From<(T, U)>
    for MeshRenderComponent
{
    fn from((x, y): (T, U)) -> Self {
        let mut m = MeshRenderComponent::simple(x);
        m.add(y);
        m
    }
}

impl<T: 'static + MeshRenderable, U: 'static + MeshRenderable, V: 'static + MeshRenderable>
    From<(T, U, V)> for MeshRenderComponent
{
    fn from((x, y, z): (T, U, V)) -> Self {
        let mut m = MeshRenderComponent::simple(x);
        m.add(y).add(z);
        m
    }
}

pub trait MeshRenderable: Send + Sync {
    fn draw(&self, trans: &Transform, transforms: &ReadStorage<Transform>, rc: &mut RenderContext);
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

impl MeshRenderable for CircleRender {
    fn draw(&self, pos: &Transform, _: &ReadStorage<Transform>, rc: &mut RenderContext) {
        rc.sr.color = self.color;
        rc.sr.set_filled(self.filled);
        rc.sr.draw_circle(pos.project(self.offset), self.radius);
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

impl MeshRenderable for RectRender {
    fn draw(&self, pos: &Transform, _: &ReadStorage<Transform>, rc: &mut RenderContext) {
        rc.sr.color = self.color;
        rc.sr.set_filled(self.filled);
        if pos.is_angle_zero() {
            rc.sr
                .draw_rect_centered(pos.get_position(), self.width, self.height)
        } else {
            rc.sr.draw_rect_cos_sin(
                pos.get_position(),
                self.width,
                self.height,
                pos.get_cos(),
                pos.get_sin(),
            )
        }
    }
}

pub struct LineToRender {
    pub to: Entity,
    pub color: Color,
}

impl MeshRenderable for LineToRender {
    fn draw(&self, trans: &Transform, transforms: &ReadStorage<Transform>, rc: &mut RenderContext) {
        let e = self.to;
        let pos2 = transforms.get(e).unwrap().get_position();
        rc.sr.color = self.color;
        rc.sr.draw_line(trans.get_position(), pos2);
    }
}

pub struct LineRender {
    pub offset: Vector2<f32>,
    pub color: Color,
}

impl MeshRenderable for LineRender {
    fn draw(&self, trans: &Transform, _: &ReadStorage<Transform>, rc: &mut RenderContext) {
        let start = trans.get_position();
        let end = start + self.offset;
        rc.sr.color = self.color;
        rc.sr.draw_line(start, end);
    }
}
