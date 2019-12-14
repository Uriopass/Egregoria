use crate::engine::components::Position;
use crate::engine::render_context::RenderContext;
use cgmath::Vector2;
use ggez::graphics::{Color, WHITE};
use specs::{Component, Entity, ReadStorage, VecStorage};

#[derive(Component)]
#[storage(VecStorage)]
pub struct MeshRender {
    pub orders: Vec<Box<dyn MeshRenderable>>,
}

impl MeshRender {
    pub fn empty() -> Self {
        MeshRender { orders: vec![] }
    }

    pub fn add<T: 'static + MeshRenderable>(&mut self, x: T) -> &mut Self {
        self.orders.push(Box::new(x));
        self
    }

    pub fn simple<T: 'static + MeshRenderable>(x: T) -> Self {
        MeshRender {
            orders: vec![Box::new(x)],
        }
    }
}

impl<T: 'static + MeshRenderable> From<T> for MeshRender {
    fn from(x: T) -> Self {
        MeshRender::simple(x)
    }
}

impl<T: 'static + MeshRenderable, U: 'static + MeshRenderable> From<(T, U)> for MeshRender {
    fn from((x, y): (T, U)) -> Self {
        let mut m = MeshRender::simple(x);
        m.add(y);
        m
    }
}

impl<T: 'static + MeshRenderable, U: 'static + MeshRenderable, V: 'static + MeshRenderable>
    From<(T, U, V)> for MeshRender
{
    fn from((x, y, z): (T, U, V)) -> Self {
        let mut m = MeshRender::simple(x);
        m.add(y).add(z);
        m
    }
}

pub trait MeshRenderable: Send + Sync {
    fn draw(&self, pos: Vector2<f32>, positions: &ReadStorage<Position>, rc: &mut RenderContext);
}

pub struct CircleRender {
    pub radius: f32,
    pub color: Color,
    pub filled: bool,
}

impl Default for CircleRender {
    fn default() -> Self {
        CircleRender {
            radius: 0.0,
            color: WHITE,
            filled: true,
        }
    }
}

impl MeshRenderable for CircleRender {
    fn draw(&self, pos: Vector2<f32>, _: &ReadStorage<Position>, rc: &mut RenderContext) {
        rc.sr.color = self.color;
        rc.sr.set_filled(self.filled);
        rc.sr.draw_circle(pos, self.radius);
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
    fn draw(&self, pos: Vector2<f32>, _: &ReadStorage<Position>, rc: &mut RenderContext) {
        rc.sr.color = self.color;
        rc.sr.set_filled(self.filled);
        rc.sr.draw_rect(
            pos - Vector2::new(self.width / 2., self.height / 2.),
            self.width,
            self.height,
        )
    }
}

pub struct LineToRender {
    pub to: Entity,
    pub color: Color,
}

impl MeshRenderable for LineToRender {
    fn draw(&self, pos: Vector2<f32>, positions: &ReadStorage<Position>, rc: &mut RenderContext) {
        let e = self.to;
        let pos2 = positions.get(e).unwrap().0;
        rc.sr.color = self.color;
        rc.sr.draw_line(pos, pos2);
    }
}

pub struct LineRender {
    pub start: Vector2<f32>,
    pub end: Vector2<f32>,
    pub color: Color,
}

impl MeshRenderable for LineRender {
    fn draw(&self, _: Vector2<f32>, _: &ReadStorage<Position>, rc: &mut RenderContext) {
        let start = self.start;
        let end = self.end;
        rc.sr.color = self.color;
        rc.sr.draw_line(start, end);
    }
}
