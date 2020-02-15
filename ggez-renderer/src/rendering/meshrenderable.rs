use crate::rendering::render_context::RenderContext;
use cgmath::Vector2;
use ggez::graphics::Color;
use scale::physics::Transform;
use scale::rendering::meshrender_component::{
    CircleRender, LineRender, LineToRender, MeshRenderEnum, RectRender,
};
use scale::specs::ReadStorage;

pub trait MeshRenderable: Send + Sync {
    fn draw(&self, trans: &Transform, transforms: &ReadStorage<Transform>, rc: &mut RenderContext);
}

impl MeshRenderable for MeshRenderEnum {
    fn draw(&self, trans: &Transform, transforms: &ReadStorage<Transform>, rc: &mut RenderContext) {
        match self {
            MeshRenderEnum::Circle(x) => x.draw(trans, transforms, rc),
            MeshRenderEnum::Rect(x) => x.draw(trans, transforms, rc),
            MeshRenderEnum::LineTo(x) => x.draw(trans, transforms, rc),
            MeshRenderEnum::Line(x) => x.draw(trans, transforms, rc),
        }
    }
}

impl MeshRenderable for CircleRender {
    fn draw(&self, pos: &Transform, _: &ReadStorage<Transform>, rc: &mut RenderContext) {
        rc.sr.color = scale_color(self.color);
        rc.sr.set_filled(self.filled);
        rc.sr.draw_circle(pos.project(self.offset), self.radius);
    }
}

impl MeshRenderable for RectRender {
    fn draw(&self, trans: &Transform, _: &ReadStorage<Transform>, rc: &mut RenderContext) {
        rc.sr.color = scale_color(self.color);
        rc.sr.set_filled(self.filled);
        if trans.is_angle_zero() {
            rc.sr
                .draw_rect_centered(trans.position(), self.width, self.height)
        } else {
            let rect_pos = trans.position()
                + Vector2::<f32>::new(
                    self.offset.x * trans.get_cos() + self.offset.y * trans.get_sin(),
                    self.offset.x * trans.get_sin() - self.offset.y * trans.get_cos(),
                );
            rc.sr.draw_rect_cos_sin(
                rect_pos,
                self.width,
                self.height,
                trans.get_cos(),
                trans.get_sin(),
            )
        }
    }
}

impl MeshRenderable for LineToRender {
    fn draw(&self, trans: &Transform, transforms: &ReadStorage<Transform>, rc: &mut RenderContext) {
        let e = self.to;
        let pos2 = transforms.get(e).unwrap().position();
        rc.sr.color = scale_color(self.color);
        rc.sr.draw_stroke(trans.position(), pos2, self.thickness);
    }
}

impl MeshRenderable for LineRender {
    fn draw(&self, trans: &Transform, _: &ReadStorage<Transform>, rc: &mut RenderContext) {
        let start = trans.position();
        let end = start + self.offset;
        rc.sr.color = scale_color(self.color);
        rc.sr.draw_stroke(start, end, self.thickness);
    }
}

pub fn scale_color(color: scale::rendering::Color) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}
