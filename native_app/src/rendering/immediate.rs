#![allow(dead_code)]

use common::{AudioKind, Z_DEBUG};
use geom::{LinearColor, Polygon, Vec2, OBB};
use wgpu_engine::{FrameContext, SpriteBatch, Tesselator};

register_resource_noserialize!(ImmediateSound);
#[derive(Default)]
pub struct ImmediateSound {
    pub orders: Vec<(&'static str, AudioKind)>,
}

impl ImmediateSound {
    pub fn play(&mut self, sound: &'static str, kind: AudioKind) {
        self.orders.push((sound, kind))
    }
}

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
    Polygon {
        poly: Polygon,
    },
    OBB(OBB),
    TexturedOBB {
        obb: OBB,
        path: String,
    },
}

#[derive(Clone)]
pub struct ImmediateOrder {
    pub kind: OrderKind,
    pub color: LinearColor,
    pub z: f32,
}

register_resource_noserialize!(ImmediateDraw);
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
    pub fn color(&mut self, col: impl Into<LinearColor>) -> &mut Self {
        self.order.color = col.into();
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
        let order = std::mem::replace(
            &mut self.order,
            ImmediateOrder {
                kind: OrderKind::Circle {
                    pos: Vec2::ZERO,
                    radius: 0.0,
                },
                color: LinearColor::TRANSPARENT,
                z: 0.0,
            },
        );
        if self.persistent {
            self.draw.persistent_orders.push(order)
        } else {
            self.draw.orders.push(order)
        }
    }
}

impl ImmediateDraw {
    fn builder(&mut self, kind: OrderKind) -> ImmediateBuilder {
        ImmediateBuilder {
            draw: self,
            order: ImmediateOrder {
                kind,
                color: LinearColor::WHITE,
                z: Z_DEBUG,
            },
            persistent: false,
        }
    }
    pub fn circle(&mut self, pos: Vec2, radius: f32) -> ImmediateBuilder {
        self.builder(OrderKind::Circle { pos, radius })
    }

    pub fn line(&mut self, from: Vec2, to: Vec2, thickness: f32) -> ImmediateBuilder {
        self.builder(OrderKind::Line {
            from,
            to,
            thickness,
        })
    }

    pub fn polyline(&mut self, points: impl Into<Vec<Vec2>>, thickness: f32) -> ImmediateBuilder {
        self.builder(OrderKind::PolyLine {
            points: points.into(),
            thickness,
        })
    }

    pub fn polygon(&mut self, poly: Polygon) -> ImmediateBuilder {
        self.builder(OrderKind::Polygon { poly })
    }

    pub fn stroke_circle(&mut self, pos: Vec2, radius: f32, thickness: f32) -> ImmediateBuilder {
        self.builder(OrderKind::StrokeCircle {
            pos,
            radius,
            thickness,
        })
    }

    pub fn obb(&mut self, obb: OBB) -> ImmediateBuilder {
        self.builder(OrderKind::OBB(obb))
    }

    pub fn textured_obb(&mut self, obb: OBB, path: String) -> ImmediateBuilder {
        self.builder(OrderKind::TexturedOBB { obb, path })
    }

    pub fn clear_persistent(&mut self) {
        self.persistent_orders.clear();
    }

    pub fn apply(&mut self, tess: &mut Tesselator, ctx: &mut FrameContext) {
        for ImmediateOrder { kind, color, z } in
            self.persistent_orders.iter().chain(self.orders.iter())
        {
            let z = *z;
            tess.set_color(*color);
            match *kind {
                OrderKind::Circle { pos, radius } => {
                    tess.draw_circle(pos, z, radius);
                }
                OrderKind::Line {
                    from,
                    to,
                    thickness,
                } => {
                    tess.draw_stroke(from, to, z, thickness);
                }
                OrderKind::StrokeCircle {
                    pos,
                    radius,
                    thickness,
                } => {
                    tess.draw_stroke_circle(pos, z, radius, thickness);
                }
                OrderKind::PolyLine {
                    ref points,
                    thickness,
                } => {
                    tess.draw_polyline(points, z, thickness);
                }
                OrderKind::Polygon { ref poly } => {
                    tess.draw_filled_polygon(poly.as_slice(), z);
                }
                OrderKind::OBB(ref obb) => {
                    let [ax1, ax2] = obb.axis();
                    tess.draw_rect_cos_sin(
                        obb.center(),
                        z,
                        ax1.magnitude(),
                        ax2.magnitude(),
                        ax1.normalize(),
                    );
                }
                OrderKind::TexturedOBB { obb, ref path } => {
                    let tex = ctx.gfx.texture(path, None);
                    ctx.objs.push(Box::new(
                        SpriteBatch::builder(tex)
                            .push(obb.center(), obb.axis()[0], z, *color, (1.0, 1.0))
                            .build(ctx.gfx)
                            .unwrap(),
                    ));
                }
            }
        }
    }
}
