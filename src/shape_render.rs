use ggez::graphics::DrawMode::Fill;
use ggez::graphics::*;
use ggez::{Context, GameError, GameResult};

pub struct ShapeRenderer {
    color: Color,
    mode: DrawMode,
    meshbuilder: Option<MeshBuilder>,
}

#[allow(dead_code)]
impl ShapeRenderer {
    pub fn new() -> ShapeRenderer {
        ShapeRenderer {
            color: WHITE,
            mode: DrawMode::fill(),
            meshbuilder: None,
        }
    }

    pub fn begin(&mut self) {
        self.meshbuilder = Some(MeshBuilder::new());
    }

    pub fn draw_circle(&mut self, x: f32, y: f32, r: f32) {
        if let Some(m) = &mut self.meshbuilder {
            m.circle(self.mode, [x, y], r, 0.5, self.color);
        }
    }

    pub fn end(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mesh: Mesh = match self.meshbuilder.take() {
            None => {
                return Err(GameError::RenderError(
                    "You have to call begin before end".parse().unwrap(),
                ))
            }
            Some(x) => x.build(ctx)?,
        };
        draw(ctx, &mesh, DrawParam::new().dest([0., 0.]))
    }
}
