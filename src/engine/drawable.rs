use ggez::GameResult;

use crate::engine::render_context::RenderContext;

pub trait Drawable {
    fn draw(ctx: &mut RenderContext) -> GameResult<()>;
}
