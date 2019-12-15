use crate::geometry::rect::Rect;
use cgmath::{ElementWise, EuclideanSpace, Point2, Vector2};
use ggez::graphics::{Color, DrawMode, MeshBuilder, Vertex, WHITE};
use nalgebra::Isometry2;
use ncollide2d::query::Proximity;
use ncollide2d::shape::Cuboid;
use ncollide2d::shape::Segment;

pub struct ShapeRenderer {
    pub color: Color,
    pub mode: DrawMode,
    pub meshbuilder: MeshBuilder,
    pub screen_box: Rect,
    pub screen_collider: (Cuboid<f32>, Isometry2<f32>),
    pub empty: bool,
    pub zoom: f32,
}

impl ShapeRenderer {
    pub fn new(screen_box: &Rect, zoom: f32) -> Self {
        ShapeRenderer {
            color: WHITE,
            mode: DrawMode::fill(),
            meshbuilder: MeshBuilder::new(),
            screen_box: screen_box.clone(),
            screen_collider: (
                Cuboid::new([screen_box.w / 2., screen_box.h / 2.].into()),
                Isometry2::new(
                    nalgebra::Vector2::new(
                        screen_box.x + screen_box.w / 2.,
                        screen_box.y + screen_box.h / 2.,
                    ),
                    nalgebra::zero(),
                ),
            ),
            empty: true,
            zoom,
        }
    }
}

#[allow(dead_code)]
impl ShapeRenderer {
    pub fn set_filled(&mut self, filled: bool) {
        if filled {
            self.mode = DrawMode::fill()
        } else {
            self.mode = DrawMode::stroke(1.);
        }
    }

    pub fn draw_circle(&mut self, p: Vector2<f32>, r: f32) {
        let pp = Point2::from_vec(p);

        if self.screen_box.contains_within(p, r) {
            self.meshbuilder.circle(self.mode, pp, r, 0.3, self.color);
            self.empty = false;
        }
    }

    pub fn draw_rect_centered(&mut self, p: Vector2<f32>, width: f32, height: f32) {
        self.meshbuilder.rectangle(
            self.mode,
            ggez::graphics::Rect::new(p.x - width / 2., p.y - height / 2., width, height),
            self.color,
        );
        self.empty = false;
    }

    pub fn draw_rect_cos_sin(
        &mut self,
        p: Vector2<f32>,
        width: f32,
        height: f32,
        cos: f32,
        sin: f32,
    ) {
        let a = Point2::new(width / 2. * cos, width / 2. * sin);
        let b = Vector2::new(height / 2. * -sin, height / 2. * cos);

        let points: [Point2<f32>; 4] = [
            a + b + p,
            a - b + p,
            a.mul_element_wise(-1.) - b + p,
            a.mul_element_wise(-1.) + b + p,
        ];

        self.meshbuilder
            .polyline(self.mode, &points, self.color)
            .expect("Error building rect");
    }

    pub fn draw_line(&mut self, p1: Vector2<f32>, p2: Vector2<f32>) {
        let zero_iso: Isometry2<f32> = nalgebra::Isometry2::new(nalgebra::zero(), nalgebra::zero());

        let segment = Segment::new(
            nalgebra::Point2::new(p1.x, p1.y),
            nalgebra::Point2::new(p2.x, p2.y),
        );

        let p = ncollide2d::query::proximity(
            &self.screen_collider.1,
            &self.screen_collider.0,
            &zero_iso,
            &segment,
            0.0,
        );

        if let Proximity::Intersecting = p {
            self.meshbuilder
                .line(
                    &[Point2::from_vec(p1), Point2::from_vec(p2)],
                    0.5 / self.zoom,
                    Color {
                        a: (self.zoom * self.zoom * 50.).min(1.).max(0.),
                        ..self.color
                    },
                )
                .expect("Line error");
            self.empty = false;
        }
    }
}
