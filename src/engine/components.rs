use crate::engine::render_context::RenderContext;
use cgmath::num_traits::zero;
use cgmath::Vector2;
use ggez::graphics::{Color, WHITE};
use ncollide2d::pipeline::CollisionObjectSlabHandle;
use specs::{Component, Entity, NullStorage, ReadStorage, VecStorage};

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position(pub Vector2<f32>);

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Kinematics {
    pub velocity: Vector2<f32>,
    pub acceleration: Vector2<f32>,
}

impl Kinematics {
    pub fn zero() -> Self {
        Kinematics {
            velocity: zero(),
            acceleration: zero(),
        }
    }

    #[allow(dead_code)]
    pub fn from_velocity(x: Vector2<f32>) -> Self {
        Kinematics {
            velocity: x,
            acceleration: zero(),
        }
    }
}

#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Collider(pub CollisionObjectSlabHandle);

#[derive(Component)]
#[storage(VecStorage)]
pub struct MeshRender {
    pub orders: Vec<Box<dyn MeshRenderable>>,
}

pub struct MeshRenderBuilder {
    mr: MeshRender,
}
impl MeshRenderBuilder {
    pub fn new() -> Self {
        MeshRenderBuilder {
            mr: MeshRender { orders: vec![] },
        }
    }

    pub fn add<T: 'static + MeshRenderable>(mut self, x: T) -> Self {
        self.mr.orders.push(Box::new(x));
        self
    }

    pub fn build(self) -> MeshRender {
        self.mr
    }

    pub fn simple<T: 'static + MeshRenderable>(x: T) -> MeshRender {
        MeshRender {
            orders: vec![Box::new(x)],
        }
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

#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Movable;
