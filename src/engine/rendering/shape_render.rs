use crate::geometry::rect::Rect;
use cgmath::{ElementWise, EuclideanSpace, Point2, Vector2};
use ggez::graphics::{Color, DrawMode, MeshBuilder, Vertex, WHITE};
use nalgebra::Isometry2;
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Cuboid, Segment};

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
                Cuboid::new([screen_box.w / 2.0, screen_box.h / 2.0].into()),
                Isometry2::new(
                    nalgebra::Vector2::new(
                        screen_box.x + screen_box.w / 2.0,
                        screen_box.y + screen_box.h / 2.0,
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
            self.mode = DrawMode::stroke(1.0);
        }
    }

    pub fn draw_circle(&mut self, p: Vector2<f32>, r: f32) {
        let pp = Point2::from_vec(p);

        if self.screen_box.contains_within(p, r) {
            self.meshbuilder
                .circle(self.mode, pp, r, 0.3 / self.zoom, self.color);
            self.empty = false;
        }
    }

    pub fn reset(&mut self) {
        self.meshbuilder = MeshBuilder::new();
        self.empty = true;
        self.color = WHITE;
        self.mode = DrawMode::fill();
    }

    pub fn draw_rect_centered(&mut self, p: Vector2<f32>, width: f32, height: f32) {
        if !self.screen_box.contains_within(p, width.max(height)) {
            return;
        }
        self.meshbuilder.rectangle(
            self.mode,
            ggez::graphics::Rect::new(p.x - width / 2.0, p.y - height / 2.0, width, height),
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
        if !self.screen_box.contains_within(p, width.max(height)) {
            return;
        }

        let a = Point2::new(width / 2.0 * cos, width / 2.0 * sin);
        let b = Vector2::new(height / 2.0 * -sin, height / 2.0 * cos);
        let points: [Point2<f32>; 4] = [
            a + b + p,
            a - b + p,
            a.mul_element_wise(-1.0) - b + p,
            a.mul_element_wise(-1.0) + b + p,
        ];

        match self.mode {
            DrawMode::Fill(_) => {
                let verts: [Vertex; 4] = [
                    Vertex {
                        pos: [points[0].x, points[0].y],
                        uv: [0.0, 0.0],
                        color: [self.color.r, self.color.g, self.color.b, self.color.a],
                    },
                    Vertex {
                        pos: [points[1].x, points[1].y],
                        uv: [1.0, 0.0],
                        color: [self.color.r, self.color.g, self.color.b, self.color.a],
                    },
                    Vertex {
                        pos: [points[2].x, points[2].y],
                        uv: [1.0, 1.0],
                        color: [self.color.r, self.color.g, self.color.b, self.color.a],
                    },
                    Vertex {
                        pos: [points[3].x, points[3].y],
                        uv: [0.0, 1.0],
                        color: [self.color.r, self.color.g, self.color.b, self.color.a],
                    },
                ];
                self.meshbuilder.raw(&verts, &[0, 1, 2, 0, 2, 3], None);
            }
            DrawMode::Stroke(_) => {
                self.meshbuilder
                    .polygon(self.mode, &points, self.color)
                    .expect("Error building rect");
            }
        }
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
                        a: (self.zoom * self.zoom * 50.0).min(1.0).max(0.0),
                        ..self.color
                    },
                )
                .expect("Line error");
            self.empty = false;
        }
    }
}
