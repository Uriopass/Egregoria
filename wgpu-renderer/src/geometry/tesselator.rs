use crate::engine::{IndexType, MeshBuilder, Vertex};
use crate::geometry::rect::Rect;
use cgmath::{vec2, InnerSpace, Vector2};
use scale::rendering::{Color, LinearColor};

pub struct Tesselator {
    pub color: LinearColor,
    pub meshbuilder: MeshBuilder,
    pub cull_rect: Option<Rect>,
    pub zoom: f32,
}

const DEFAULT_THICKNESS: f32 = 0.2;
impl Tesselator {
    pub fn new(cull_rect: Option<Rect>, zoom: f32) -> Self {
        Tesselator {
            color: LinearColor::WHITE,
            meshbuilder: MeshBuilder::new(),
            cull_rect,
            zoom,
        }
    }
}

#[allow(dead_code)]
impl Tesselator {
    pub fn draw_circle(&mut self, p: Vector2<f32>, z: f32, r: f32) -> bool {
        if r <= 0.0 || self.cull_rect.map_or(false, |x| !x.contains_within(p, r)) {
            return false;
        }

        let n_points = ((6.0 * (r * self.zoom).cbrt()) as usize).max(4);
        let n_pointsu32 = n_points as u32;

        let color = self.color.into();

        self.meshbuilder.extend_with(|vertices, index_push| {
            vertices.push(Vertex {
                position: p.extend(z).into(),
                color,
            });

            for i in 0..n_pointsu32 {
                let v = std::f32::consts::PI * 2.0 * (i as f32) / n_points as f32;
                let trans = r * vec2(v.cos(), v.sin());
                vertices.push(Vertex {
                    position: (p + trans).extend(z).into(),
                    color,
                });
                index_push(0);
                index_push(i + 1);
                if i == n_pointsu32 - 1 {
                    index_push(1);
                } else {
                    index_push(i + 2);
                }
            }
        });

        true
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color.into();
    }

    pub fn reset(&mut self) {
        self.meshbuilder = MeshBuilder::new();
        self.color = LinearColor::WHITE;
    }

    pub fn draw_rect_cos_sin(
        &mut self,
        p: Vector2<f32>,
        z: f32,
        width: f32,
        height: f32,
        cos_sin: Vector2<f32>,
    ) -> bool {
        if self
            .cull_rect
            .map_or(false, |x| x.contains_within(p, width.max(height)))
        {
            return false;
        }

        let a = (width / 2.0) * cos_sin;
        let b = (height / 2.0) * vec2(-cos_sin.y, cos_sin.x);
        let pxy = vec2(p.x, p.y);

        let points: [Vector2<_>; 4] = [a + b + pxy, a - b + pxy, -a - b + pxy, -a + b + pxy];

        let color: [f32; 4] = self.color.into();

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
        if self.cull_rect.map_or(false, |x| {
            !x.intersects_line_within(p1, p2, thickness * 0.5)
        }) {
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

        let color: [f32; 4] = self.color.into();

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

    pub fn draw_polyline(
        &mut self,
        points: &[Vector2<f32>],
        first_dir: Vector2<f32>,
        last_dir: Vector2<f32>,
        z: f32,
        thickness: f32,
    ) -> bool {
        let n_points = points.len();
        if n_points < 2 || thickness <= 0.0 {
            return true;
        }
        if n_points == 2 {
            self.draw_stroke(points[0], points[1], z, thickness);
            return true;
        }
        if let Some(cull_rect) = self.cull_rect {
            let window_intersects =
                |x: &[Vector2<f32>]| cull_rect.intersects_line_within(x[0], x[1], thickness);

            if !points.windows(2).any(window_intersects) {
                return false;
            }
        }

        let halfthick = thickness * 0.5;

        let color = self.color.into();

        let verts = &mut self.meshbuilder.vertices;
        let indices = &mut self.meshbuilder.indices;
        let offset = verts.len() as IndexType;

        let nor: Vector2<f32> = halfthick * vec2(-first_dir.y, first_dir.x);

        verts.push(Vertex {
            position: (points[0] + nor).extend(z).into(),
            color,
        });

        verts.push(Vertex {
            position: (points[0] - nor).extend(z).into(),
            color,
        });

        for (index, window) in points.windows(3).enumerate() {
            let a = window[0];
            let elbow = window[1];
            let c = window[2];

            let ac = c - a;

            let dist = ac.magnitude();

            if dist <= 0.0 {
                continue;
            }

            let nor = (halfthick / dist) * vec2(-ac.y, ac.x);
            verts.push(Vertex {
                position: (elbow + nor).extend(z).into(),
                color,
            });
            verts.push(Vertex {
                position: (elbow - nor).extend(z).into(),
                color,
            });
            let i = index as u32;
            indices.push(offset + i * 2);
            indices.push(offset + i * 2 + 1);
            indices.push(offset + i * 2 + 2);

            indices.push(offset + i * 2 + 1);
            indices.push(offset + i * 2 + 2);
            indices.push(offset + i * 2 + 3);
        }

        let nor: Vector2<f32> = halfthick * vec2(-last_dir.y, last_dir.x);

        verts.push(Vertex {
            position: (points[n_points - 1] + nor).extend(z).into(),
            color,
        });

        verts.push(Vertex {
            position: (points[n_points - 1] - nor).extend(z).into(),
            color,
        });

        let i = (n_points * 2) as u32;
        indices.push(offset + i - 3);
        indices.push(offset + i - 2);
        indices.push(offset + i - 1);

        indices.push(offset + i - 4);
        indices.push(offset + i - 3);
        indices.push(offset + i - 2);

        true
    }

    pub fn draw_line(&mut self, p1: Vector2<f32>, p2: Vector2<f32>, z: f32) -> bool {
        self.draw_stroke(p1, p2, z, 1.5 / self.zoom)
    }

    pub fn draw_grid(&mut self, grid_size: f32, color: Color) {
        let screen = self
            .cull_rect
            .expect("Cannot draw grid when not culling since I do not know where is the screen");

        let mut x = (screen.x / grid_size).ceil() * grid_size;
        self.set_color(color);
        while x < screen.x + screen.w {
            self.draw_line(
                Vector2::new(x, screen.y),
                Vector2::new(x, screen.y + screen.h),
                0.01,
            );
            x += grid_size;
        }

        let mut y = (screen.y / grid_size).ceil() * grid_size;
        while y < screen.y + screen.h {
            self.draw_line(
                Vector2::new(screen.x, y),
                Vector2::new(screen.x + screen.w, y),
                0.01,
            );
            x += grid_size;
            y += grid_size;
        }
    }
}
