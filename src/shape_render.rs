use ggez::graphics::Vertex;
use ggez::graphics::*;
use ggez::{Context, GameResult};

pub struct ShapeRenderer {
    pub color: Color,
    pub mode: DrawMode,
    meshbuilder: MeshBuilder,
    screen_box: Rect,
    once: bool,
}

const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

#[allow(dead_code)]
impl ShapeRenderer {
    pub fn begin(mut rect: Rect) -> ShapeRenderer {
        rect.scale(1.1, 1.1);
        rect.x -= 50.;
        rect.y -= 50.;
        rect.w += 100.;
        rect.h += 100.;
        ShapeRenderer {
            color: WHITE,
            mode: DrawMode::fill(),
            meshbuilder: MeshBuilder::new(),
            once: false,
            screen_box: rect,
        }
    }
    pub fn draw_circle(&mut self, p: impl Into<mint::Point2<f32>>, r: f32) {
        let point = p.into();
        if self.screen_box.contains(point) {
            self.meshbuilder
                .circle(self.mode, point, r, 0.3, self.color);
            self.once = true;
        }
    }

    pub fn draw_rect(&mut self, p: impl Into<mint::Point2<f32>>, width: f32, height: f32) {
        let v = p.into();
        self.meshbuilder
            .rectangle(self.mode, Rect::new(v.x, v.y, width, height), self.color);
        self.once = true;
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
        self.once = true;
    }

    pub fn end(self, ctx: &mut Context) -> GameResult<()> {
        if !self.once {
            return Ok(());
        }
        let mesh = self.meshbuilder.build(ctx)?;
        draw(ctx, &mesh, DrawParam::new().dest([0., 0.]))
    }
}
