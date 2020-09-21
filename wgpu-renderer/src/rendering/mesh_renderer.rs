use crate::geometry::Tesselator;
use egregoria::legion::IntoQuery;
use egregoria::rendering::meshrender_component::{MeshRender, MeshRenderEnum};
use egregoria::Egregoria;
use geom::Transform;

pub struct MeshRenderer;

impl MeshRenderer {
    pub fn render(goria: &mut Egregoria, tess: &mut Tesselator) {
        for (trans, mr) in <(&Transform, &MeshRender)>::query().iter(&goria.world) {
            if mr.hide {
                continue;
            }
            for order in &mr.orders {
                match order {
                    MeshRenderEnum::StrokeCircle(x) => {
                        tess.color = x.color.into();
                        tess.draw_stroke_circle(
                            trans.project(x.offset),
                            mr.z,
                            x.radius,
                            x.thickness,
                        );
                    }
                    MeshRenderEnum::Circle(x) => {
                        tess.color = x.color.into();
                        tess.draw_circle(trans.project(x.offset), mr.z, x.radius);
                    }
                    MeshRenderEnum::Rect(x) => {
                        tess.color = x.color.into();
                        let rect_pos = trans.position() + trans.apply_rotation(x.offset);
                        tess.draw_rect_cos_sin(
                            rect_pos,
                            mr.z,
                            x.width,
                            x.height,
                            trans.direction(),
                        );
                    }
                    MeshRenderEnum::LineTo(x) => {
                        tess.color = x.color.into();
                        let e = x.to;
                        if let Some(pos_to) = goria.pos(e) {
                            tess.draw_stroke(trans.position(), pos_to, mr.z, x.thickness);
                        }
                    }
                    MeshRenderEnum::Line(x) => {
                        tess.color = x.color.into();
                        let start = trans.position();
                        let end = start + x.offset;
                        tess.draw_stroke(start, end, mr.z, x.thickness);
                    }

                    MeshRenderEnum::AbsoluteLine(x) => {
                        tess.color = x.color.into();
                        tess.draw_stroke(x.src, x.dst, mr.z, x.thickness);
                    }
                }
            }
        }
    }
}
