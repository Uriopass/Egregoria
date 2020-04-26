use crate::engine::{MeshBuilder, Vertex};
use crate::geometry::rect::Rect;
use cgmath::{vec2, InnerSpace, Vector2};
use scale::rendering::Color;

pub struct Tesselator {
    pub color: Color,
    pub meshbuilder: MeshBuilder,
    pub screen_box: Rect,
    pub zoom: f32,
    pub cull: bool,
}

const DEFAULT_THICKNESS: f32 = 0.2;
impl Tesselator {
    pub fn new(screen_box: Rect, zoom: f32, cull: bool) -> Self {
        Tesselator {
            color: Color::WHITE,
            meshbuilder: MeshBuilder::new(),
            screen_box,
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
    pub fn draw_circle(&mut self, p: Vector2<f32>, z: f32, r: f32) -> bool {
        if r <= 0.0 || (self.cull && !self.screen_box.contains_within(p, r)) {
            return false;
        }

        let n_points = ((r * self.zoom) as usize).max(4);
        let n_pointsu32 = n_points as u32;

        let color = self.color.into();
        let mut points = Vec::with_capacity(n_points);
        points.push(Vertex {
            position: p.extend(z).into(),
            color,
        });

        let mut indices = Vec::with_capacity(n_points * 3);

        for i in 0..n_pointsu32 {
            let v = std::f32::consts::PI * 2.0 * (i as f32) / n_points as f32;
            let trans = r * vec2(v.cos(), v.sin());
            points.push(Vertex {
                position: (p + trans).extend(z).into(),
                color,
            });
            indices.push(0);
            indices.push(i + 1);
            if i == n_pointsu32 - 1 {
                indices.push(1);
            } else {
                indices.push(i + 2);
            }
        }

        self.meshbuilder.extend(&points, &indices);
        true
    }

    pub fn reset(&mut self) {
        self.meshbuilder = MeshBuilder::new();
        self.color = Color::WHITE;
    }

    pub fn draw_rect_cos_sin(
        &mut self,
        p: Vector2<f32>,
        z: f32,
        width: f32,
        height: f32,
        cos_sin: Vector2<f32>,
    ) -> bool {
        if self.cull && !self.screen_box.contains_within(p, width.max(height)) {
            return false;
        }

        let a = (width / 2.0) * cos_sin;
        let b = (height / 2.0) * vec2(-cos_sin.y, cos_sin.x);
        let pxy = vec2(p.x, p.y);

        let points: [Vector2<_>; 4] = [a + b + pxy, a - b + pxy, -a - b + pxy, -a + b + pxy];

        let mut color: [f32; 4] = self.color.into();

        for x in color.iter_mut() {
            *x = from_srgb(*x);
        }

        let verts: [Vertex; 4] = [
            Vertex {
                position: [points[0].x, points[0].y, z],
                color,
            },
            Vertex {
                position: [points[1].x, points[1].y, z],
                color,
            },
            Vertex {
                position: [points[2].x, points[2].y, z],
                color,
            },
            Vertex {
                position: [points[3].x, points[3].y, z],
                color,
            },
        ];
        self.meshbuilder.extend(&verts, &[0, 1, 2, 0, 2, 3]);
        true
    }

    pub fn draw_stroke(
        &mut self,
        p1: Vector2<f32>,
        p2: Vector2<f32>,
        z: f32,
        thickness: f32,
    ) -> bool {
        if self.cull
            && !self
                .screen_box
                .intersects_line_within(p1, p2, thickness * 0.5)
        {
            return false;
        }

        let diff = p2 - p1;
        let dist = diff.magnitude();
        if dist < 1e-5 {
            return false;
        }
        let ratio = (thickness * 0.5) / dist;
        let nor: Vector2<f32> = ratio * vec2(-diff.y, diff.x);

        let points: [Vector2<f32>; 4] = [p1 - nor, p1 + nor, p2 + nor, p2 - nor];

        let mut color: [f32; 4] = self.color.into();

        for x in color.iter_mut() {
            *x = from_srgb(*x);
        }

        let verts: [Vertex; 4] = [
            Vertex {
                position: points[0].extend(z).into(),
                color,
            },
            Vertex {
                position: points[1].extend(z).into(),
                color,
            },
            Vertex {
                position: points[2].extend(z).into(),
                color,
            },
            Vertex {
                position: points[3].extend(z).into(),
                color,
            },
        ];
        self.meshbuilder.extend(&verts, &[0, 1, 2, 0, 2, 3]);
        true
    }

    pub fn draw_polyline(&mut self, points: &[Vector2<f32>], z: f32, thickness: f32) -> bool {
        if self.cull {
            let window_intersects = |x: &[Vector2<f32>]| {
                self.screen_box.intersects_line_within(
                    vec2(x[0].x, x[0].y),
                    vec2(x[1].x, x[1].y),
                    thickness,
                )
            };
            if !points.windows(2).any(window_intersects) {
                return false;
            }
        }

        for w in points.windows(2) {
            self.draw_stroke(w[0], w[1], z, thickness);
        }
        true
    }

    pub fn draw_line(&mut self, p1: Vector2<f32>, p2: Vector2<f32>, z: f32) -> bool {
        self.draw_stroke(p1, p2, z, 1.0 / self.zoom)
    }
}
