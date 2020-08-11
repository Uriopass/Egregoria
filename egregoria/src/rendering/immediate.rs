pub use crate::rendering::Color;
use geom::Vec2;

#[derive(Copy, Clone)]
pub enum ImmediateOrder {
    Circle { pos: Vec2, size: f32 },
    Line { from: Vec2, to: Vec2 },
}

#[derive(Default)]
pub struct ImmediateDraw {
    pub orders: Vec<(ImmediateOrder, Color)>,
    pub persistent_orders: Vec<(ImmediateOrder, Color)>,
}

pub struct ImmediateBuilder<'a> {
    draw: &'a mut ImmediateDraw,
    order: ImmediateOrder,
    color: Color,
    persistent: bool,
}

impl<'a> ImmediateBuilder<'a> {
    pub fn color(&mut self, col: Color) -> &mut Self {
        self.color = col;
        self
    }

    pub fn persistent(&mut self) -> &mut Self {
        self.persistent = true;
        self
    }
}

impl<'a> Drop for ImmediateBuilder<'a> {
    fn drop(&mut self) {
        if self.persistent {
            self.draw.persistent_orders.push((self.order, self.color))
        } else {
            self.draw.orders.push((self.order, self.color))
        }
    }
}

impl ImmediateDraw {
    pub fn circle(&mut self, pos: Vec2, size: f32) -> ImmediateBuilder {
        ImmediateBuilder {
            draw: self,
            order: ImmediateOrder::Circle { pos, size },
            color: Color::WHITE,
            persistent: false,
        }
    }

    pub fn line(&mut self, from: Vec2, to: Vec2) -> ImmediateBuilder {
        ImmediateBuilder {
            draw: self,
            order: ImmediateOrder::Line { from, to },
            color: Color::WHITE,
            persistent: false,
        }
    }

    pub fn clear_persistent(&mut self) {
        self.persistent_orders.clear();
    }
}
