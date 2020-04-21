use crate::geometry::tesselator::Tesselator;
use crate::rendering::camera_handler;
use crate::rendering::camera_handler::CameraHandler;
use cgmath::{EuclideanSpace, Point2, Vector2};
use ggez::graphics::{Color, DrawParam, Font, Image, Mesh, Text};
use ggez::{graphics, Context, GameResult};

pub struct RenderContext<'a> {
    pub cam: &'a mut camera_handler::CameraHandler,
    pub tess: Tesselator,
    font: Option<Font>,
    pub ctx: &'a mut Context,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        cam: &'a mut CameraHandler,
        ctx: &'a mut Context,
        font: Option<Font>,
    ) -> RenderContext<'a> {
        let rect = cam.get_screen_box();
        let tess = Tesselator::new(rect, cam.camera.zoom, true);
        RenderContext {
            ctx,
            cam,
            tess,
            font,
        }
    }

    pub fn clear(&mut self) {
        graphics::clear(self.ctx, graphics::Color::from_rgb(0, 0, 0));
    }

    #[allow(dead_code)]
    pub fn draw_grid(&mut self, grid_size: f32, color: Color) {
        let screen = self.tess.screen_box;

        let mut x = (screen.x / grid_size).ceil() * grid_size;
        self.tess.color = color;
        while x < screen.x + screen.w {
            self.tess.draw_line(
                Vector2::new(x, screen.y),
                Vector2::new(x, screen.y + screen.h),
            );
            x += grid_size;
        }

        let mut y = (screen.y / grid_size).ceil() * grid_size;
        while y < screen.y + screen.h {
            self.tess.draw_line(
                Vector2::new(screen.x, y),
                Vector2::new(screen.x + screen.w, y),
            );
            x += grid_size;
            y += grid_size;
        }
    }

    #[allow(dead_code)]
    pub fn draw_text(
        &mut self,
        text: &str,
        mut pos: Vector2<f32>,
        size: f32,
        color: Color,
    ) -> GameResult<()> {
        let text = Text::new((text, self.font.unwrap(), 70.0));
        pos.y += text.height(self.ctx) as f32 * 0.02 * size;
        let trans = graphics::DrawParam::new()
            .color(color)
            .dest(Point2::from_vec(pos))
            .scale([0.02 * size, -0.02 * size]);
        graphics::draw(self.ctx, &text, trans)
    }

    #[allow(dead_code)]
    pub fn draw_image<P>(&mut self, image: &Image, mut pos: Vector2<f32>) -> GameResult<()> {
        pos.y += image.height() as f32;
        let trans = graphics::DrawParam::new()
            .dest(Point2::from_vec(pos))
            .scale([1.0, -1.0]);
        graphics::draw(self.ctx, image, trans)
    }

    #[allow(dead_code)]
    pub fn draw_mesh(&mut self, mesh: &Mesh, dp: DrawParam) -> GameResult<()> {
        graphics::draw(self.ctx, mesh, dp)
    }

    pub fn flush(&mut self) -> GameResult<()> {
        if !self.tess.empty {
            let mesh = self.tess.meshbuilder.build(self.ctx)?;
            graphics::draw(self.ctx, &mesh, DrawParam::new().dest([0.0, 0.0]))?;
            self.tess.reset();
        }
        Ok(())
    }

    pub fn finish(mut self) -> GameResult<()> {
        self.flush()
    }
}
