use ggez::graphics::Vertex;
use ggez::graphics::*;
use ggez::{Context, GameResult};

pub struct ShapeRenderer {
    pub color: Color,
    pub mode: DrawMode,
    meshbuilder: MeshBuilder,
}
const QUAD_VERTS: [Vertex; 4] = [
    Vertex {
        pos: [0.0, 0.0],
        uv: [0.0, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, 0.0],
        uv: [1.0, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [1.0, 1.0],
        uv: [1.0, 1.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
    Vertex {
        pos: [0.0, 1.0],
        uv: [0.0, 1.0],
        color: [1.0, 1.0, 1.0, 1.0],
    },
];

const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

#[allow(dead_code)]
impl ShapeRenderer {
    pub fn begin() -> ShapeRenderer {
        ShapeRenderer {
            color: WHITE,
            mode: DrawMode::fill(),
            meshbuilder: MeshBuilder::new(),
        }
    }
    pub fn draw_circle(&mut self, p: impl Into<mint::Point2<f32>>, r: f32) {
        self.meshbuilder.circle(self.mode, p, r, 0.5, self.color);
    }

    pub fn draw_rect(&mut self, p: impl Into<mint::Point2<f32>>, width: f32, height: f32) {
        let v = p.into();
        self.meshbuilder
            .rectangle(self.mode, Rect::new(v.x, v.y, width, height), self.color);
    }

    pub fn draw_rect_skinny(&mut self, p: impl Into<mint::Point2<f32>>, width: f32, height: f32) {
        let v = p.into();
        self.meshbuilder.raw(&QUAD_VERTS, &QUAD_INDICES, None);
    }

    pub fn end(self, ctx: &mut Context) -> GameResult<()> {
        let mesh = self.meshbuilder.build(ctx)?;
        draw(ctx, &mesh, DrawParam::new().dest([0., 0.]))
    }
}
