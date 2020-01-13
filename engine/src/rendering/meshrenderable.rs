use crate::rendering::render_context::RenderContext;
use ggez::graphics::Color;
use scale::engine_interaction::Transform;
use scale::rendering::meshrender_component::{
    CircleRender, LineRender, LineToRender, MeshRenderEnum, RectRender,
};

use specs::ReadStorage;

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
    fn draw(&self, pos: &Transform, _: &ReadStorage<Transform>, rc: &mut RenderContext) {
        rc.sr.color = scale_color(self.color);
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

impl MeshRenderable for LineToRender {
    fn draw(&self, trans: &Transform, transforms: &ReadStorage<Transform>, rc: &mut RenderContext) {
        let e = self.to;
        let pos2 = transforms.get(e).unwrap().get_position();
        rc.sr.color = scale_color(self.color);
        rc.sr.draw_line(trans.get_position(), pos2);
    }
}

impl MeshRenderable for LineRender {
    fn draw(&self, trans: &Transform, _: &ReadStorage<Transform>, rc: &mut RenderContext) {
        let start = trans.get_position();
        let end = start + self.offset;
        rc.sr.color = scale_color(self.color);
        rc.sr.draw_line(start, end);
    }
}

fn scale_color(color: scale::rendering::Color) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}
