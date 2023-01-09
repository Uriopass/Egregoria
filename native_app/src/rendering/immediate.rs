#![allow(dead_code)]

use common::{AudioKind, FastMap};
use geom::{LinearColor, Polygon, Vec2, Vec3, AABB, OBB};
use wgpu_engine::meshload::load_mesh;
use wgpu_engine::{FrameContext, InstancedMeshBuilder, MeshInstance, SpriteBatch, Tesselator};

#[derive(Default)]
pub(crate) struct ImmediateSound {
    pub(crate) orders: Vec<(&'static str, AudioKind)>,
}

impl ImmediateSound {
    pub(crate) fn play(&mut self, sound: &'static str, kind: AudioKind) {
        self.orders.push((sound, kind))
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone)]
pub(crate) enum OrderKind {
    Circle {
        pos: Vec3,
        radius: f32,
    },
    StrokeCircle {
        pos: Vec3,
        radius: f32,
        thickness: f32,
    },
    Line {
        from: Vec3,
        to: Vec3,
        thickness: f32,
    },
    PolyLine {
        points: Vec<Vec3>,
        thickness: f32,
        loops: bool,
    },
    Polygon {
        poly: Polygon,
        z: f32,
    },
    OBB {
        obb: OBB,
        z: f32,
    },
    TexturedOBB {
        obb: OBB,
        path: String,
        z: f32,
    },
    Mesh {
        path: String,
        pos: Vec3,
        dir: Vec3,
    },
}

#[derive(Clone)]
pub(crate) struct ImmediateOrder {
    pub(crate) kind: OrderKind,
    pub(crate) color: LinearColor,
}

#[derive(Default)]
pub(crate) struct ImmediateDraw {
    pub(crate) orders: Vec<ImmediateOrder>,
    pub(crate) persistent_orders: Vec<ImmediateOrder>,
    pub(crate) mesh_cache: FastMap<String, InstancedMeshBuilder>,
}

pub(crate) struct ImmediateBuilder<'a> {
    draw: &'a mut ImmediateDraw,
    order: ImmediateOrder,
    persistent: bool,
}

impl<'a> ImmediateBuilder<'a> {
    pub(crate) fn color(&mut self, col: impl Into<LinearColor>) -> &mut Self {
        self.order.color = col.into();
        self
    }

    pub(crate) fn persistent(&mut self) -> &mut Self {
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
                    pos: Vec3::ZERO,
                    radius: 0.0,
                },
                color: LinearColor::TRANSPARENT,
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
    fn builder(&mut self, kind: OrderKind) -> ImmediateBuilder<'_> {
        ImmediateBuilder {
            draw: self,
            order: ImmediateOrder {
                kind,
                color: LinearColor::WHITE,
            },
            persistent: false,
        }
    }
    pub(crate) fn circle(&mut self, pos: Vec3, radius: f32) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::Circle { pos, radius })
    }

    pub(crate) fn line(&mut self, from: Vec3, to: Vec3, thickness: f32) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::Line {
            from,
            to,
            thickness,
        })
    }

    pub(crate) fn polyline(
        &mut self,
        points: impl Into<Vec<Vec3>>,
        thickness: f32,
        loops: bool,
    ) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::PolyLine {
            points: points.into(),
            thickness,
            loops,
        })
    }

    pub(crate) fn polygon(&mut self, poly: Polygon, z: f32) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::Polygon { poly, z })
    }

    pub(crate) fn stroke_circle(
        &mut self,
        pos: Vec3,
        radius: f32,
        thickness: f32,
    ) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::StrokeCircle {
            pos,
            radius,
            thickness,
        })
    }

    pub(crate) fn obb(&mut self, obb: OBB, z: f32) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::OBB { obb, z })
    }

    pub(crate) fn aabb(&mut self, aabb: AABB, z: f32) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::OBB {
            obb: OBB::new(aabb.center(), Vec2::X, aabb.w(), aabb.h()),
            z,
        })
    }

    pub(crate) fn textured_obb(&mut self, obb: OBB, path: String, z: f32) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::TexturedOBB { obb, path, z })
    }

    pub(crate) fn mesh(&mut self, path: String, pos: Vec3, dir: Vec3) -> ImmediateBuilder<'_> {
        self.builder(OrderKind::Mesh { path, pos, dir })
    }

    pub(crate) fn clear_persistent(&mut self) {
        self.persistent_orders.clear();
    }

    pub(crate) fn apply(&mut self, tess: &mut Tesselator, ctx: &mut FrameContext<'_>) {
        for ImmediateOrder { kind, color } in
            self.persistent_orders.iter().chain(self.orders.iter())
        {
            tess.set_color(*color);
            match *kind {
                OrderKind::Circle { pos, radius } => {
                    tess.draw_circle(pos, radius);
                }
                OrderKind::Line {
                    from,
                    to,
                    thickness,
                } => {
                    tess.draw_stroke(from, to, thickness);
                }
                OrderKind::StrokeCircle {
                    pos,
                    radius,
                    thickness,
                } => {
                    tess.draw_stroke_circle(pos, radius, thickness);
                }
                OrderKind::PolyLine {
                    ref points,
                    thickness,
                    loops,
                } => {
                    tess.draw_polyline(points, thickness, loops);
                }
                OrderKind::Polygon { ref poly, z } => {
                    tess.draw_filled_polygon(poly.as_slice(), z);
                }
                OrderKind::OBB { ref obb, z } => {
                    let [ax1, ax2] = obb.axis();
                    tess.draw_rect_cos_sin(
                        obb.center().z(z),
                        ax1.mag(),
                        ax2.mag(),
                        ax1.normalize(),
                    );
                }
                OrderKind::TexturedOBB { obb, ref path, z } => {
                    let tex = unwrap_cont!(ctx.gfx.try_texture(path, "some immediate obb"));
                    ctx.objs.push(Box::new(
                        SpriteBatch::builder(tex)
                            .push(obb.center().z(z), obb.axis()[0].z0(), *color, (1.0, 1.0))
                            .build(ctx.gfx)
                            .unwrap(),
                    ));
                }
                OrderKind::Mesh { ref path, pos, dir } => {
                    let m = self.mesh_cache.get_mut(path);
                    let i = if let Some(x) = m {
                        x
                    } else {
                        self.mesh_cache.insert(
                            path.clone(),
                            InstancedMeshBuilder::new(load_mesh(ctx.gfx, path).unwrap()),
                        );
                        self.mesh_cache.get_mut(path).unwrap()
                    };

                    i.instances.push(MeshInstance {
                        pos,
                        dir,
                        tint: color.a(1.0),
                    });

                    ctx.objs.push(Box::new(i.build(ctx.gfx).unwrap()))
                }
            }
        }
        for v in self.mesh_cache.values_mut() {
            if let Some(x) = v.build(ctx.gfx) {
                ctx.objs.push(Box::new(x));
            }
            v.instances.clear();
        }
    }
}
