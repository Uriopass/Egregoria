use crate::geometry::rect::Rect;
use cgmath::{vec2, InnerSpace};
use cgmath::{ElementWise, EuclideanSpace, Point2, Vector2};
use ggez::graphics::{Color, DrawMode, LineJoin, MeshBuilder, StrokeOptions, Vertex, WHITE};

pub struct Tesselator {
    pub color: Color,
    pub mode: DrawMode,
    pub meshbuilder: MeshBuilder,
    pub screen_box: Rect,
    pub empty: bool,
    pub zoom: f32,
    pub cull: bool,
}

const DEFAULT_THICKNESS: f32 = 0.2;
impl Tesselator {
    pub fn new(screen_box: Rect, zoom: f32, cull: bool) -> Self {
        Tesselator {
            color: WHITE,
            mode: DrawMode::fill(),
            meshbuilder: MeshBuilder::new(),
            screen_box,
            empty: true,
            zoom,
            cull,
        }
    }
}

fn from_srgb(component: f32) -> f32 {
    let a = 0.055;
    if component <= 0.04045 {
        component / 12.92
    } else {
        ((component + a) / (1.0 + a)).powf(2.4)
    }
}

#[allow(dead_code)]
impl Tesselator {
    pub fn set_filled(&mut self, filled: bool) {
        if filled {
            self.mode = DrawMode::fill()
        } else {
            self.mode = DrawMode::stroke(DEFAULT_THICKNESS)
        }
    }

    pub fn draw_circle(&mut self, p: Vector2<f32>, r: f32) -> bool {
        let pp = Point2::from_vec(p);

        if !self.cull || (r > 0.0 && self.screen_box.contains_within(p, r)) {
            self.meshbuilder
                .circle(self.mode, pp, r, 0.3 / self.zoom, self.color);

            self.empty = false;
            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.meshbuilder = MeshBuilder::new();
        self.empty = true;
        self.color = WHITE;
        self.mode = DrawMode::fill();
    }

    pub fn draw_rect_cos_sin_uv(
        &mut self,
        p: Vector2<f32>,
        width: f32,
        height: f32,
        cos_sin: Vector2<f32>,
        uv_from: Vector2<f32>,
        uv_to: Vector2<f32>,
    ) -> bool {
        if self.cull && !self.screen_box.contains_within(p, width.max(height)) {
            return false;
        }

        let a = Point2::new(width / 2.0 * cos_sin.x, width / 2.0 * cos_sin.y);
        let b = Vector2::new(height / 2.0 * -cos_sin.y, height / 2.0 * cos_sin.x);
        let points: [Point2<f32>; 4] = [
            a + b + p,
            a - b + p,
            a.mul_element_wise(-1.0) - b + p,
            a.mul_element_wise(-1.0) + b + p,
        ];

        let col = Color::new(
            from_srgb(self.color.r),
            from_srgb(self.color.g),
            from_srgb(self.color.b),
            1.0,
        );
        match self.mode {
            DrawMode::Fill(_) => {
                let verts: [Vertex; 4] = [
                    Vertex {
                        pos: [points[0].x, points[0].y],
                        uv: [uv_from.x, uv_from.y],
                        color: [col.r, col.g, col.b, col.a],
                    },
                    Vertex {
                        pos: [points[1].x, points[1].y],
                        uv: [uv_to.x, uv_from.y],
                        color: [col.r, col.g, col.b, col.a],
                    },
                    Vertex {
                        pos: [points[2].x, points[2].y],
                        uv: [uv_to.x, uv_to.y],
                        color: [col.r, col.g, col.b, col.a],
                    },
                    Vertex {
                        pos: [points[3].x, points[3].y],
                        uv: [uv_from.x, uv_to.y],
                        color: [col.r, col.g, col.b, col.a],
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
        self.empty = false;
        true
    }

    pub fn draw_raw_quad(&mut self, points: [Vector2<f32>; 4], uv: [Vector2<f32>; 4]) -> bool {
        if self.cull && !self.screen_box.contains_within(points[0], 10.0) {
            return false;
        }

        let verts: [Vertex; 4] = [
            Vertex {
                pos: [points[0].x, points[0].y],
                uv: [uv[0].x, uv[0].y],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [points[1].x, points[1].y],
                uv: [uv[1].x, uv[1].y],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [points[2].x, points[2].y],
                uv: [uv[2].x, uv[2].y],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                pos: [points[3].x, points[3].y],
                uv: [uv[3].x, uv[3].y],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];
        self.meshbuilder.raw(&verts, &[0, 1, 2, 0, 2, 3], None);

        self.empty = false;
        true
    }

    pub fn draw_rect_cos_sin(
        &mut self,
        p: Vector2<f32>,
        width: f32,
        height: f32,
        cos_sin: Vector2<f32>,
    ) -> bool {
        self.draw_rect_cos_sin_uv(p, width, height, cos_sin, vec2(0.0, 0.0), vec2(0.0, 0.0))
    }

    pub fn draw_stroke(&mut self, p1: Vector2<f32>, p2: Vector2<f32>, thickness: f32) -> bool {
        if self.cull
            && !self
                .screen_box
                .intersects_line_within(p1, p2, thickness / 2.0)
        {
            return false;
        }

        let diff = p2 - p1;
        let dist = diff.magnitude();
        if dist < 1e-5 {
            return false;
        }
        let nor: Vector2<f32> = ((thickness * 0.5) / dist) * vec2(-diff.y, diff.x);

        let points: [Vector2<f32>; 4] = [p1 - nor, p1 + nor, p2 + nor, p2 - nor];

        let col = Color::new(
            from_srgb(self.color.r),
            from_srgb(self.color.g),
            from_srgb(self.color.b),
            self.color.a,
        );

        let verts: [Vertex; 4] = [
            Vertex {
                pos: [points[0].x, points[0].y],
                uv: [0.0, 0.0],
                color: [col.r, col.g, col.b, col.a],
            },
            Vertex {
                pos: [points[1].x, points[1].y],
                uv: [0.0, 0.0],
                color: [col.r, col.g, col.b, col.a],
            },
            Vertex {
                pos: [points[2].x, points[2].y],
                uv: [0.0, 0.0],
                color: [col.r, col.g, col.b, col.a],
            },
            Vertex {
                pos: [points[3].x, points[3].y],
                uv: [0.0, 0.0],
                color: [col.r, col.g, col.b, col.a],
            },
        ];
        self.meshbuilder.raw(&verts, &[0, 1, 2, 0, 2, 3], None);
        self.empty = false;
        true
    }

    pub fn draw_polyline(&mut self, points: &[Vector2<f32>], thickness: f32) -> bool {
        if self.cull {
            let window_intersects = |x: &[Vector2<f32>]| {
                self.screen_box
                    .intersects_line_within(x[0], x[1], thickness)
            };
            if !points.windows(2).any(window_intersects) {
                return false;
            }
        }

        self.meshbuilder
            .polyline(
                DrawMode::Stroke(
                    StrokeOptions::default()
                        .with_line_width(thickness)
                        .with_line_join(LineJoin::Round),
                ),
                &points
                    .iter()
                    .map(|x| Point2::new(x.x, x.y))
                    .collect::<Vec<_>>(),
                Color {
                    a: (self.zoom * self.zoom * 50.0).min(self.color.a).max(0.0),
                    ..self.color
                },
            )
            .expect("Line error");
        self.empty = false;
        true
    }

    pub fn draw_line(&mut self, p1: Vector2<f32>, p2: Vector2<f32>) -> bool {
        self.draw_stroke(p1, p2, 0.5 / self.zoom)
    }
}
