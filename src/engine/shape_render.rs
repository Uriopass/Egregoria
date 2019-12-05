use ggez::graphics::Vertex;
use ggez::graphics::*;

pub struct ShapeRenderer {
    pub color: Color,
    pub mode: DrawMode,
    pub meshbuilder: MeshBuilder,
    pub screen_box: Rect,
    pub empty: bool,
}

const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

#[allow(dead_code)]
impl ShapeRenderer {
    pub fn draw_circle(&mut self, p: impl Into<mint::Point2<f32>>, r: f32) {
        let point = p.into();
        if self.screen_box.contains(point) {
            self.meshbuilder
                .circle(self.mode, point, r, 0.3, self.color);
            self.empty = false;
        }
    }

    pub fn draw_rect(&mut self, p: impl Into<mint::Point2<f32>>, width: f32, height: f32) {
        let v = p.into();
        self.meshbuilder
            .rectangle(self.mode, Rect::new(v.x, v.y, width, height), self.color);
        self.empty = false;
    }

    pub fn draw_rect_skinny(&mut self, p: impl Into<mint::Point2<f32>>, width: f32, height: f32) {
        let v = p.into();

        let verts: [Vertex; 4] = [
            Vertex {
                pos: [v.x, v.y],
                uv: [0.0, 0.0],
                color: [self.color.r, self.color.g, self.color.b, self.color.a],
            },
            Vertex {
                pos: [v.x + width, v.y],
                uv: [1.0, 0.0],
                color: [self.color.r, self.color.g, self.color.b, self.color.a],
            },
            Vertex {
                pos: [v.x + width, v.y + height],
                uv: [1.0, 1.0],
                color: [self.color.r, self.color.g, self.color.b, self.color.a],
            },
            Vertex {
                pos: [v.x, v.y + height],
                uv: [0.0, 1.0],
                color: [self.color.r, self.color.g, self.color.b, self.color.a],
            },
        ];

        self.meshbuilder.raw(&verts, &QUAD_INDICES, None);
        self.empty = false;
    }
}
