use cgmath::{EuclideanSpace, Point2, Vector2};
use ggez::graphics::{DrawMode, DrawParam, Image, Mesh, MeshBuilder, WHITE};
use ggez::{graphics, Context, GameResult};

use crate::engine::camera_handler;
use crate::engine::camera_handler::CameraHandler;
use crate::engine::shape_render::ShapeRenderer;

pub struct RenderContext<'a> {
    pub cam: &'a mut camera_handler::CameraHandler,
    pub sr: ShapeRenderer,
    ctx: &'a mut Context,
}

impl<'a> RenderContext<'a> {
    pub fn new(cam: &'a mut CameraHandler, ctx: &'a mut Context) -> RenderContext<'a> {
        let rect = cam.get_screen_box();
        let sr = ShapeRenderer {
            screen_box: rect,
            zoom: cam.camera.zoom,
            ..Default::default()
        };
        RenderContext { ctx, cam, sr }
    }

    pub fn clear(&mut self) {
        graphics::clear(self.ctx, graphics::Color::from_rgb(0, 0, 0));
        graphics::set_window_title(
            self.ctx,
            format!("{} FPS", ggez::timer::fps(self.ctx) as i32).as_ref(),
        );
    }

    pub fn draw_text(&mut self, text: &graphics::Text, mut pos: Vector2<f32>) -> GameResult<()> {
        pos.y += text.height(self.ctx) as f32;
        let trans = graphics::DrawParam::new()
            .dest(Point2::from_vec(pos))
            .scale([1., -1.]);
        graphics::draw(self.ctx, text, trans)
    }

    #[allow(dead_code)]
    pub fn draw_image<P>(&mut self, image: &Image, mut pos: Vector2<f32>) -> GameResult<()> {
        pos.y += image.height() as f32;
        let trans = graphics::DrawParam::new()
            .dest(Point2::from_vec(pos))
            .scale([1., -1.]);
        graphics::draw(self.ctx, image, trans)
    }

    #[allow(dead_code)]
    pub fn draw_mesh(&mut self, mesh: &Mesh, dp: DrawParam) -> GameResult<()> {
        graphics::draw(self.ctx, mesh, dp)
    }

    pub fn finish(self) -> GameResult<()> {
        if !self.sr.empty {
            let mesh = self.sr.meshbuilder.build(self.ctx)?;
            graphics::draw(self.ctx, &mesh, DrawParam::new().dest([0.0, 0.0]))
        } else {
            Ok(())
        }
    }
}
