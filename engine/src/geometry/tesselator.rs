use itertools::Itertools;

use geom::{vec3, Intersect, LinearColor, Segment, Vec2, Vec3, AABB};

use crate::geometry::earcut::earcut;
use crate::{IndexType, MeshVertex};

pub struct Tesselator<'a> {
    pub color: LinearColor,
    pub vertices: &'a mut Vec<MeshVertex>,
    pub indices: &'a mut Vec<IndexType>,
    pub cull_rect: Option<AABB>,
    pub zoom: f32,
    pub normal: Vec3,
}

impl<'a> Tesselator<'a> {
    pub fn new(
        vertices: &'a mut Vec<MeshVertex>,
        indices: &'a mut Vec<IndexType>,
        cull_rect: Option<AABB>,
        zoom: f32,
    ) -> Self {
        Tesselator {
            color: LinearColor::BLACK,
            vertices,
            indices,
            cull_rect,
            zoom,
            normal: Vec3::Z,
        }
    }

    pub fn extend_with(&mut self, f: impl FnOnce(&mut Vec<MeshVertex>, &mut dyn FnMut(IndexType))) {
        let offset = self.vertices.len() as IndexType;
        let vertices = &mut self.vertices;
        let indices = &mut self.indices;
        let mut x = move |index: IndexType| {
            indices.push(index + offset);
        };
        f(vertices, &mut x);
    }

    pub fn draw_circle(&mut self, p: Vec3, r: f32) -> bool {
        if r <= 0.0
            || self
                .cull_rect
                .map_or(false, |x| !x.contains_within(p.xy(), r))
        {
            return false;
        }
        let n_points = ((6.0 * (r * self.zoom).cbrt()) as usize).max(4);

        self.draw_regular_polygon(p, r, n_points, 0.0)
    }

    pub fn draw_regular_polygon(
        &mut self,
        p: Vec3,
        r: f32,
        n_points: usize,
        start_angle: f32,
    ) -> bool {
        if r <= 0.0
            || self
                .cull_rect
                .map_or(false, |x| !x.contains_within(p.xy(), r))
        {
            return false;
        }

        let color = self.color.into();
        let n_pointsu32 = n_points as u32;
        let normal = self.normal;

        self.extend_with(|vertices, index_push| {
            vertices.push(MeshVertex {
                position: p.into(),
                color,
                normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            });

            for i in 0..n_pointsu32 {
                let v = std::f32::consts::PI * 2.0 * (i as f32) / n_points as f32 + start_angle;
                let trans = p + r * vec3(v.cos(), v.sin(), 0.0);
                vertices.push(MeshVertex {
                    position: trans.into(),
                    color,
                    normal,
                    uv: [0.0; 2],
                    tangent: [0.0; 4],
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

    pub fn draw_filled_polygon(&mut self, points: &[Vec2], z: f32) -> bool {
        let oob = self.cull_rect.map_or(false, |x| {
            !points.iter().any(|&p| x.contains_within(p, 1.0))
        });
        if oob {
            return false;
        }

        let color: [f32; 4] = self.color.into();
        let normal = self.normal;
        self.extend_with(|vertices, index_push| {
            vertices.extend(points.iter().map(|p| MeshVertex {
                position: [p.x, p.y, z],
                color,
                normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            }));

            earcut(points, &[], |x, y, z| {
                index_push(x as u32);
                index_push(y as u32);
                index_push(z as u32);
            });
        });

        true
    }

    pub fn draw_stroke_circle(&mut self, p: Vec3, r: f32, thickness: f32) -> bool {
        if r <= 0.0
            || self
                .cull_rect
                .map_or(false, |x| !x.contains_within(p.xy(), r))
        {
            return false;
        }

        let halfthick = thickness * 0.5;
        let n_points = ((6.0 * (r * self.zoom).cbrt()) as usize).max(4);
        let n_pointsu32 = n_points as u32;

        let color = self.color.into();
        let normal = self.normal;
        self.extend_with(|vertices, index_push| {
            vertices.push(MeshVertex {
                position: (p + Vec3::x(r + halfthick)).into(),
                color,
                normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            });
            vertices.push(MeshVertex {
                position: (p + Vec3::x(r - halfthick)).into(),
                color,
                normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            });

            for i in 0..n_pointsu32 {
                let v = std::f32::consts::PI * 2.0 * (i as f32) / n_points as f32;
                let trans = vec3(v.cos(), v.sin(), 0.0);
                let p1 = p + (r + halfthick) * trans;
                let p2 = p + (r - halfthick) * trans;
                vertices.push(MeshVertex {
                    position: p1.into(),
                    color,
                    normal,
                    uv: [0.0; 2],
                    tangent: [0.0; 4],
                });
                vertices.push(MeshVertex {
                    position: p2.into(),
                    color,
                    normal,
                    uv: [0.0; 2],
                    tangent: [0.0; 4],
                });
                index_push(i * 2 + 2);
                index_push(i * 2 + 1);
                index_push(i * 2);

                index_push(i * 2 + 1);
                index_push(i * 2 + 2);
                index_push(i * 2 + 3);
            }

            let i = n_pointsu32;

            index_push(0);
            index_push(i * 2 + 1);
            index_push(i * 2);

            index_push(i * 2 + 1);
            index_push(0);
            index_push(1);
        });
        true
    }

    pub fn set_color(&mut self, color: impl Into<LinearColor>) {
        self.color = color.into();
    }

    pub fn draw_rect_cos_sin(&mut self, p: Vec3, width: f32, height: f32, cos_sin: Vec2) -> bool {
        if let Some(x) = self.cull_rect {
            if !x.contains_within(p.xy(), width.max(height)) {
                return false;
            }
        }

        let cos_sin = cos_sin.z0();
        let a = (width * 0.5) * cos_sin;
        let b = (height * 0.5) * -cos_sin.perp_up();

        let points: [Vec3; 4] = [a + b + p, a - b + p, -a - b + p, -a + b + p];

        let color: [f32; 4] = self.color.into();

        let verts: [MeshVertex; 4] = [
            MeshVertex {
                position: points[0].into(),
                color,
                normal: self.normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            },
            MeshVertex {
                position: points[1].into(),
                color,
                normal: self.normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            },
            MeshVertex {
                position: points[2].into(),
                color,
                normal: self.normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            },
            MeshVertex {
                position: points[3].into(),
                color,
                normal: self.normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            },
        ];
        self.extend_with(|v, add_i| {
            v.extend_from_slice(&verts);
            add_i(2);
            add_i(1);
            add_i(0);
            add_i(3);
            add_i(2);
            add_i(0);
        });
        true
    }

    pub fn draw_stroke(&mut self, p1: Vec3, p2: Vec3, thickness: f32) -> bool {
        let perp = (p2 - p1)
            .xy()
            .perpendicular()
            .try_normalize()
            .unwrap_or(Vec2::X);
        self.draw_stroke_full(p1, p2, perp, thickness)
    }

    pub fn draw_stroke_full(&mut self, p1: Vec3, p2: Vec3, dir: Vec2, thickness: f32) -> bool {
        if let Some(x) = self.cull_rect {
            if !x
                .expand(thickness * 0.5)
                .intersects(&Segment::new(p1.xy(), p2.xy()))
            {
                return false;
            }
        }

        let dir = (thickness * 0.5) * dir.z0();
        let points: [Vec3; 4] = [p1 - dir, p1 + dir, p2 + dir, p2 - dir];

        let color: [f32; 4] = self.color.into();

        let verts: [MeshVertex; 4] = [
            MeshVertex {
                position: points[0].into(),
                color,
                normal: self.normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            },
            MeshVertex {
                position: points[1].into(),
                color,
                normal: self.normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            },
            MeshVertex {
                position: points[2].into(),
                color,
                normal: self.normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            },
            MeshVertex {
                position: points[3].into(),
                color,
                normal: self.normal,
                uv: [0.0; 2],
                tangent: [0.0; 4],
            },
        ];

        let n_inv = self.normal.z < 0.0;
        self.extend_with(|v, add_i| {
            v.extend_from_slice(&verts);
            if n_inv {
                add_i(2);
                add_i(1);
                add_i(0);
                add_i(3);
                add_i(2);
                add_i(0);
            } else {
                add_i(0);
                add_i(1);
                add_i(2);
                add_i(0);
                add_i(2);
                add_i(3);
            }
        });
        true
    }
    pub fn draw_polyline_with_dir(
        &mut self,
        points: &[Vec3],
        first_dir: Vec2,
        last_dir: Vec2,
        thickness: f32,
    ) -> bool {
        self.draw_polyline_full(points.iter().copied(), first_dir, last_dir, thickness, 0.0)
    }

    pub fn draw_polyline_full(
        &mut self,
        mut points: impl ExactSizeIterator<Item = Vec3> + Clone,
        first_dir: Vec2,
        last_dir: Vec2,
        thickness: f32,
        offset: f32,
    ) -> bool {
        let n_points = points.len();
        if n_points < 2 || thickness <= 0.0 {
            return true;
        }
        if n_points == 2 {
            let first: Vec3 = points.next().unwrap();
            let second: Vec3 = points.next().unwrap();
            let dir = (first - second)
                .try_normalize()
                .unwrap_or_default()
                .perp_up();
            return self.draw_stroke_full(
                first + dir * offset,
                second + dir * offset,
                first_dir.perpendicular(),
                thickness,
            );
        }
        if let Some(cull_rect) = self.cull_rect {
            let window_intersects = |(a, b): (Vec3, Vec3)| {
                cull_rect
                    .expand(thickness)
                    .intersects(&Segment::new(a.xy(), b.xy()))
            };

            if !points.clone().tuple_windows().any(window_intersects) {
                return false;
            }
        }

        let halfthick = thickness * 0.5;

        let color = self.color.into();
        let normal = self.normal;
        let swap = (self.normal.z < 0.0) as u32 * 2;
        self.extend_with(move |verts, index_push| {
            let mut idx_quad = move |index| {
                index_push(index * 2 + swap);
                index_push(index * 2 + 1);
                index_push(index * 2 + 2 - swap);

                index_push(index * 2 + 3 - swap);
                index_push(index * 2 + 2);
                index_push(index * 2 + 1 + swap);
            };

            let mut pvert = move |pos: Vec3| {
                verts.push(MeshVertex {
                    position: pos.into(),
                    color,
                    normal,
                    uv: [0.0; 2],
                    tangent: [0.0; 4],
                });
            };

            let mut index: u32 = 0;
            for (a, elbow, c) in points.tuple_windows() {
                let a: Vec3 = a;
                let elbow: Vec3 = elbow;
                let c: Vec3 = c;
                if index == 0 {
                    let nor = -first_dir.perpendicular();
                    pvert(a + (nor * (offset + halfthick)).z0());
                    pvert(a + (nor * (offset - halfthick)).z0());
                }

                let ae = unwrap_or!((elbow - a).xy().try_normalize(), continue);
                let ce = unwrap_or!((elbow - c).xy().try_normalize(), continue);

                let dir = match (ae + ce).try_normalize() {
                    Some(x) => {
                        let d = ae.perp_dot(ce);

                        if d.abs() < 0.01 {
                            -ae.perpendicular()
                        } else if d < 0.0 {
                            -x
                        } else {
                            x
                        }
                    }
                    None => -ae.perpendicular(),
                };

                let mut sin_theta = ae.perp_dot(dir);

                if sin_theta < 0.1 {
                    sin_theta = 0.1;
                }

                //let mul = 1.0 + (1.0 + ae.dot(ce).min(0.0)) * (std::f32::consts::SQRT_2 - 1.0);
                let mul = 1.0 / sin_theta;

                let p1 = elbow + (mul * dir * (offset + halfthick)).z0();
                let p2 = elbow + (mul * dir * (offset - halfthick)).z0();
                pvert(p1);
                pvert(p2);
                idx_quad(index);

                index += 1;
                if index as usize == n_points - 2 {
                    let nor = -last_dir.perpendicular();

                    let p1 = c + ((offset + halfthick) * nor).z0();
                    let p2 = c + ((offset - halfthick) * nor).z0();
                    pvert(p1);
                    pvert(p2);
                    idx_quad(index);
                }
            }
        });
        true
    }

    pub fn draw_polyline(&mut self, points: &[Vec3], thickness: f32, loops: bool) -> bool {
        let n_points = points.len();
        if n_points < 2 || thickness <= 0.0 {
            return true;
        }
        if n_points == 2 || (loops && n_points == 3) {
            self.draw_stroke(points[0], points[1], thickness);
            return true;
        }
        if loops {
            let elbow = points[0];
            let a = points[1];
            let c = points[points.len() - 2];

            let ae = unwrap_or!((elbow - a).xy().try_normalize(), return false);
            let dir = -ae
                .try_bisect((elbow - c).xy())
                .unwrap_or(-ae.perpendicular())
                .perpendicular();

            return self.draw_polyline_with_dir(points, dir, dir, thickness);
        }

        let first_dir = (points[1] - points[0]).normalize();
        let n = points.len();
        let last_dir = (points[n - 1] - points[n - 2]).normalize();

        self.draw_polyline_with_dir(points, first_dir.xy(), last_dir.xy(), thickness)
    }

    pub fn draw_line(&mut self, p1: Vec3, p2: Vec3) -> bool {
        self.draw_stroke(p1, p2, 1.5 / self.zoom)
    }
}
