use crate::geometry::rect;
use crate::geometry::rect::Rect;
use cgmath::{EuclideanSpace, Point2, Vector2};
use ggez::graphics::{Color, DrawMode, MeshBuilder};

pub struct ShapeRenderer {
    pub color: Color,
    pub mode: DrawMode,
    pub meshbuilder: MeshBuilder,
    pub screen_box: Rect,
    pub empty: bool,
}

#[allow(dead_code)]
impl ShapeRenderer {
    pub fn draw_circle(&mut self, p: Vector2<f32>, r: f32) {
        let pp = Point2::from_vec(p);

        if self.screen_box.contains_within(p, r) {
            self.meshbuilder.circle(self.mode, pp, r, 0.3, self.color);
            self.empty = false;
        }
    }

    pub fn draw_rect(&mut self, p: Vector2<f32>, width: f32, height: f32) {
        self.meshbuilder.rectangle(
            self.mode,
            ggez::graphics::Rect::new(p.x, p.y, width, height),
            self.color,
        );
        self.empty = false;
    }

    pub fn draw_line(&mut self, p1: Vector2<f32>, p2: Vector2<f32>) {
        if self.screen_box.contains(p1) || self.screen_box.contains(p2) {
            self.meshbuilder
                .line(
                    &[Point2::from_vec(p1), Point2::from_vec(p2)],
                    1.0,
                    self.color,
                )
                .expect("Line error");
            self.empty = false;
        }
    }
}
