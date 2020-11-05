use geom::{Color, Vec2};
use std::mem::MaybeUninit;

#[derive(Clone)]
pub enum OrderKind {
    Circle {
        pos: Vec2,
        radius: f32,
    },
    StrokeCircle {
        pos: Vec2,
        radius: f32,
        thickness: f32,
    },
    Line {
        from: Vec2,
        to: Vec2,
        thickness: f32,
    },
    PolyLine {
        points: Vec<Vec2>,
        thickness: f32,
    },
}

#[derive(Clone)]
pub struct ImmediateOrder {
    pub kind: OrderKind,
    pub color: Color,
    pub z: f32,
}

#[derive(Default)]
pub struct ImmediateDraw {
    pub orders: Vec<ImmediateOrder>,
    pub persistent_orders: Vec<ImmediateOrder>,
}

pub struct ImmediateBuilder<'a> {
    draw: &'a mut ImmediateDraw,
    order: ImmediateOrder,
    persistent: bool,
}

impl<'a> ImmediateBuilder<'a> {
    pub fn color(&mut self, col: Color) -> &mut Self {
        self.order.color = col;
        self
    }

    pub fn z(&mut self, z: f32) -> &mut Self {
        self.order.z = z;
        self
    }

    pub fn persistent(&mut self) -> &mut Self {
        self.persistent = true;
        self
    }
}

impl<'a> Drop for ImmediateBuilder<'a> {
    fn drop(&mut self) {
        let order = std::mem::replace(&mut self.order, unsafe {
            MaybeUninit::zeroed().assume_init()
        });
        if self.persistent {
            self.draw.persistent_orders.push(order)
        } else {
            self.draw.orders.push(order)
        }
    }
}

impl ImmediateDraw {
    pub fn circle(&mut self, pos: Vec2, radius: f32) -> ImmediateBuilder {
        ImmediateBuilder {
            draw: self,
            order: ImmediateOrder {
                kind: OrderKind::Circle { pos, radius },
                color: Color::WHITE,
                z: 3.0,
            },
            persistent: false,
        }
    }

    pub fn line(&mut self, from: Vec2, to: Vec2, thickness: f32) -> ImmediateBuilder {
        ImmediateBuilder {
            draw: self,
            order: ImmediateOrder {
                kind: OrderKind::Line {
                    from,
                    to,
                    thickness,
                },
                color: Color::WHITE,
                z: 3.0,
            },
            persistent: false,
        }
    }

    pub fn polyline(&mut self, points: impl Into<Vec<Vec2>>, thickness: f32) -> ImmediateBuilder {
        ImmediateBuilder {
            draw: self,
            order: ImmediateOrder {
                kind: OrderKind::PolyLine {
                    points: points.into(),
                    thickness,
                },
                color: Color::WHITE,
                z: 3.0,
            },
            persistent: false,
        }
    }

    pub fn stroke_circle(&mut self, pos: Vec2, radius: f32, thickness: f32) -> ImmediateBuilder {
        ImmediateBuilder {
            draw: self,
            order: ImmediateOrder {
                kind: OrderKind::StrokeCircle {
                    pos,
                    radius,
                    thickness,
                },
                color: Color::WHITE,
                z: 3.0,
            },
            persistent: false,
        }
    }

    pub fn clear_persistent(&mut self) {
        self.persistent_orders.clear();
    }
}
